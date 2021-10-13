#!/bin/bash -x

set -e -o pipefail

usage() {
  cat << EOT
Usage: $0 <<BUILD ID>>
Reset fractals mainnet, example BUILD ID = e1ca4db_20211011_1204
EOT
}

subsitute_node_build_id() {
  ssh $1 "PERL_BADLANG=0 perl -pi -e \"s/\\\\w+_\\\\d+_\\\\d+/$2/\" docker-compose.yml"
}

# /p2p/12D3KooWB3ENNs5vK5HNJTHVd33wEZnXdoTFwHkEUcFdQ6JxRhRJ
subsitute_bootnode_id() {
  ssh $1 "PERL_BADLANG=0 perl -pi -e \"s|/p2p/\w+|/p2p/$2|\" docker-compose.yml"
}

(( $# < 1 )) && { echo "1 arguements is required."; usage; exit 1; }

NODE_BUILD_ID=$1

node_boot="ubuntu@node-boot.mainnet.fractalprotocol.com"
node_01="ubuntu@node-1.mainnet.fractalprotocol.com"
node_02="ubuntu@node-2.mainnet.fractalprotocol.com"

exec_on_all_nodes() {
  ssh $node_boot "$1"
  ssh $node_01 "$1"
  ssh $node_02 "$1"
}

# close down nodes
exec_on_all_nodes 'docker-compose down'

# remove base-path
exec_on_all_nodes 'sudo rm -fR base-path'
exec_on_all_nodes 'sudo rm  fclMainnet*.json'

# generate new Genesis Spec
scp reset/rebuild-spec.sh $node_boot:/tmp/rebuild-spec.sh
ssh $node_boot "/tmp/rebuild-spec.sh $NODE_BUILD_ID"

# distrbute the specs, to the other nodes
ssh $node_boot 'tar cz --exclude base-path fcl*.json' | ssh $node_01 'tar xz'
ssh $node_boot 'tar cz --exclude base-path fcl*.json' | ssh $node_02 'tar xz'

# now change the compose files with the new node-build-id
subsitute_node_build_id $node_boot $NODE_BUILD_ID
subsitute_node_build_id $node_01 $NODE_BUILD_ID
subsitute_node_build_id $node_02 $NODE_BUILD_ID

# reboot $node_boot and find the bootnode id
# this by greping for the folowing and exiting the process on the pattern
# Local node identity is: (\w+)
ssh $node_boot 'docker-compose up -d'
bootnode_id=$(ssh $node_boot "docker-compose logs | grep 'Local node identity is' | awk '{print \$10}'")

echo "Found bootnode_id $bootnode_id"

# change the docker-compose files splice in the new node-build-id for $node_01 and $node_02
# run the insert-keys step for all the nodes
subsitute_bootnode_id $node_01 $bootnode_id
subsitute_bootnode_id $node_02 $bootnode_id

ssh $node_01 'docker-compose up -d'
ssh $node_02 'docker-compose up -d'

# now shoot in keys and reboot
echo "Upload block authoring and finality keys"

read -p "Press enter to continue and reboot all the nodes"

# reboot the nodes so the validator keys get activated
exec_on_all_nodes 'docker-compose restart'
