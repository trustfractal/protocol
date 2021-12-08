#!/bin/bash

set -eo pipefail

usage() {
  cat << EOT
Reset Fractal Mainnet.

Example:
$0 e1ca4db_20211011_1204
EOT
}

exec_on_authoring_nodes() {
  ssh $node_boot "$1"
  ssh $authoring_node_1 "$1"
  ssh $authoring_node_2 "$1"
}

(( $# < 1 )) && { echo "1 arguements is required."; usage; exit 1; }

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
ROOT_DIR="$SCRIPT_DIR/../.."
INFRA_DIR="$ROOT_DIR/infra/net"
AUTHORING_DIR="$INFRA_DIR/files/authoring"

DOCKER_IMAGE_ID=$1

node_boot="ubuntu@node-boot.mainnet.fractalprotocol.com"
authoring_node_1="ubuntu@node-1.mainnet.fractalprotocol.com"
authoring_node_2="ubuntu@node-2.mainnet.fractalprotocol.com"

confirm_with_user() {
  echo "!!!!!!! WARNING !!!!!!!"
  echo "THIS WILL COMPLETELY DESTROY THE STATE OF THE MAINNET BLOCKCHAIN"
  echo "Are you sure you want to continue"

  read -p "Only 'reset mainnet state' will continue: " response
  if [ "$response" != "reset mainnet state" ]; then
    exit 0
  fi
}

clear_existing_state() {
  echo "Bringing down nodes"
  exec_on_authoring_nodes 'docker-compose down'

  echo "Removing stored state"
  exec_on_authoring_nodes 'sudo rm -fR base-path'
  echo "Removing specs"
  exec_on_authoring_nodes 'sudo rm -f fclMainnet*.json'
}

create_genesis_spec() {
  echo "Rebuilding spec"
  scp $SCRIPT_DIR/rebuild-spec.sh $node_boot:/tmp/rebuild-spec.sh
  ssh $node_boot "/tmp/rebuild-spec.sh $DOCKER_IMAGE_ID"

  echo "Downloading built spec"
  ssh $node_boot 'tar cz --exclude base-path fcl*.json' | tar xz -C $INFRA_DIR/files/spec/

  echo "Copying spec to authoring nodes"
  (cd $INFRA_DIR/files/spec && tar cz *) | ssh $authoring_node_1 'tar xz'
  (cd $INFRA_DIR/files/spec && tar cz *) | ssh $authoring_node_2 'tar xz'
}

get_bootnode_peer_id() {
  ssh $node_boot "docker-compose logs | head -n100 | grep 'Local node identity is' | awk '{print \$10}' | head -n1"
}

replace_build_id() {
  sed -i "s/boymaas\/nodefcl:.*/boymaas\/nodefcl:$DOCKER_IMAGE_ID/" $1
}

replace_p2p_bootnode_id() {
  sed -i "s/p2p\/.*/p2p\/$(get_bootnode_peer_id)/" $1
}

start_boot_node() {
  echo "Starting boot node"
  replace_build_id $AUTHORING_DIR/boot-docker-compose.yml
  scp $AUTHORING_DIR/boot-docker-compose.yml $node_boot:docker-compose.yml
  ssh $node_boot 'EXTRA_ARGS="--rpc-methods=Unsafe --unsafe-rpc-external" docker-compose up -d'
}

start_authoring_nodes() {
  echo "Starting authoring nodes"
  replace_build_id $AUTHORING_DIR/node-docker-compose.yml
  replace_p2p_bootnode_id $AUTHORING_DIR/node-docker-compose.yml

  scp $AUTHORING_DIR/node-docker-compose.yml $authoring_node_1:docker-compose.yml
  ssh $authoring_node_1 'EXTRA_ARGS="--rpc-methods=Unsafe --unsafe-rpc-external" docker-compose up -d'

  scp $AUTHORING_DIR/node-docker-compose.yml $authoring_node_2:docker-compose.yml
  ssh $authoring_node_2 'EXTRA_ARGS="--rpc-methods=Unsafe --unsafe-rpc-external" docker-compose up -d'
}

wait_for_key_upload() {
  echo "Upload block authoring and finality keys now..."
  read -p "Enter when complete..."
}

restart_authoring_nodes() {
  echo "Restarting boot and authoring nodes"
  exec_on_authoring_nodes 'docker-compose restart'
}

copy_state_to_local_dir() {
  echo "Copying autoscaling state to local repository"
  ssh $node_boot 'tar cz --exclude base-path fcl*.json' | tar xz -C $INFRA_DIR/files/spec/

  replace_build_id $INFRA_DIR/files/asg/docker-compose.yml
  replace_p2p_bootnode_id $INFRA_DIR/files/asg/docker-compose.yml
}

build_ami() {
  echo "Building AMI"
  (cd $INFRA_DIR/packer && packer build -force fcl-asg-node.pkr.hcl | tee /tmp/packer_$DOCKER_IMAGE_ID.out)
  ami_id=$(cat /tmp/packer_$DOCKER_IMAGE_ID.out | grep -A 1 'AMIs were created' | tail -n1 | cut -d' ' -f2)
  sed -i "s/ami-\w*/$ami_id/" $INFRA_DIR/variables.tf
}

apply_terraform() {
  echo "Applying terraform"
  (cd $INFRA_DIR && terraform apply -auto-approve)
}

confirm_with_user
clear_existing_state
create_genesis_spec
start_boot_node
start_authoring_nodes
wait_for_key_upload
restart_authoring_nodes

copy_state_to_local_dir
build_ami
apply_terraform
