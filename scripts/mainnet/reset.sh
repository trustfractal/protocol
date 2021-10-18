#!/bin/bash

set -exo pipefail

usage() {
  cat << EOT
Reset Fractal Mainnet.

Example:
$0 e1ca4db_20211011_1204
EOT
}

exec_on_authoring_nodes() {
  ssh $node_boot "$1"
  ssh $authoring_node_01 "$1"
  ssh $authoring_node_02 "$1"
}

(( $# < 1 )) && { echo "1 arguements is required."; usage; exit 1; }

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
ROOT_DIR="$SCRIPT_DIR/../.."
INFRA_DIR="$ROOT_DIR/infra/net"

DOCKER_IMAGE_ID=$1

node_boot="ubuntu@node-boot.mainnet.fractalprotocol.com"
authoring_node_01="ubuntu@node-1.mainnet.fractalprotocol.com"
authoring_node_02="ubuntu@node-2.mainnet.fractalprotocol.com"

clear_existing_state() {
  exec_on_authoring_nodes 'docker-compose down'

  exec_on_authoring_nodes 'sudo rm -fR base-path'
  exec_on_authoring_nodes 'sudo rm -f fclMainnet*.json'
}

create_genesis_spec() {
  scp $SCRIPT_DIR/rebuild-spec.sh $node_boot:/tmp/rebuild-spec.sh
  ssh $node_boot "/tmp/rebuild-spec.sh $DOCKER_IMAGE_ID"

  ssh $node_boot 'tar cz --exclude base-path fcl*.json' | ssh $authoring_node_01 'tar xz'
  ssh $node_boot 'tar cz --exclude base-path fcl*.json' | ssh $authoring_node_02 'tar xz'
}

get_bootnode_peer_id() {
  ssh $node_boot "docker-compose logs | grep 'Local node identity is' | awk '{print \$10}' | head -n1"
}

build_id_sub_script() {
  echo "PERL_BADLANG=0 perl -pi -e \"s/\\\\w+_\\\\d+_\\\\d+/$1/\" $2"
}

substitute_node_build_id() {
  ssh $1 "$(build_id_sub_script $2 docker-compose.yml)"
}

bootnode_sub_script() {
  echo "PERL_BADLANG=0 perl -pi -e \"s|/p2p/\w+|/p2p/$1|\" $2"
}

# Replaces /p2p/<some_id> with /p2p/$2 in $1's docker-compose.yml
substitute_bootnode_peer_id() {
  ssh $1 "$(bootnode_sub_script $2 docker-compose.yml)"
}

start_authoring_nodes() {
  # Change the compose files with the new node-build-id
  substitute_node_build_id $node_boot $DOCKER_IMAGE_ID
  substitute_node_build_id $authoring_node_01 $DOCKER_IMAGE_ID
  substitute_node_build_id $authoring_node_02 $DOCKER_IMAGE_ID

  # Reboot $node_boot and find the bootnode id
  # this by greping for the folowing and exiting the process on the pattern
  # Local node identity is: (\w+)
  ssh $node_boot 'docker-compose up -d'
  bootnode_peer_id=$(get_bootnode_peer_id)

  echo "bootnode_peer_id: $bootnode_peer_id"

  # change the docker-compose files splice in the new node-build-id for $authoring_node_01 and $authoring_node_02
  # run the insert-keys step for all the nodes
  substitute_bootnode_peer_id $authoring_node_01 $bootnode_peer_id
  substitute_bootnode_peer_id $authoring_node_02 $bootnode_peer_id

  ssh $authoring_node_01 'docker-compose up -d'
  ssh $authoring_node_02 'docker-compose up -d'
}

wait_for_key_upload() {
  echo "Upload block authoring and finality keys now..."
  read -p "Enter when complete..."
}

restart_authoring_nodes() {
  exec_on_authoring_nodes 'docker-compose restart'
}

copy_state_to_local_dir() {
  ssh $node_boot 'tar cz --exclude base-path fcl*.json' | tar xz -C $INFRA_DIR/files/spec/

  sh -c "$(bootnode_sub_script $(get_bootnode_peer_id) $INFRA_DIR/files/asg/docker-compose.yml)"
  sh -c "$(build_id_sub_script $DOCKER_IMAGE_ID $INFRA_DIR/files/asg/docker-compose.yml)"
}

build_ami() {
  (cd $INFRA_DIR/packer && packer build -force fcl-asg-node.pkr.hcl | tee /tmp/packer_$DOCKER_IMAGE_ID.out)
  ami_id=$(cat /tmp/packer_$DOCKER_IMAGE_ID.out | grep -A 1 'AMIs were created' | tail -n1 | cut -d' ' -f2)
  sed -i "s/ami-\w*/$ami_id/" $INFRA_DIR/variables.tf
}

apply_terraform() {
  (cd $INFRA_DIR && terraform apply -auto-approve)
}

clear_existing_state
create_genesis_spec
start_authoring_nodes
wait_for_key_upload
restart_authoring_nodes

copy_state_to_local_dir
build_ami
apply_terraform
