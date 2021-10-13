#!/bin/bash -x

usage() {
  cat << EOT
Usage: $0 <<BUILD ID>>
Reset fractals mainnet, example BUILD ID = e1ca4db_20211011_1204
EOT
}

subsitute_node_build_id() {
  ssh $1 "PERL_BADLANG=0 perl -pi -e \"s/\\\\w+_\\\\d+_\\\\d+/$2/\" docker-compose.yml"
}

(( $# < 1 )) && { echo "1 arguements is required."; usage; exit 1; }

NODE_BUILD_ID=$1

node_00="ubuntu@node-boot.mainnet.fractalprotocol.com"
node_01="ubuntu@node-1.mainnet.fractalprotocol.com"
node_02="ubuntu@node-2.mainnet.fractalprotocol.com"

exec_on_all_nodes() {
  ssh $node_00 "$1"
  ssh $node_01 "$1"
  ssh $node_02 "$1"
}

# close down nodes
exec_on_all_nodes 'docker-compose down'

# remove base-path
exec_on_all_nodes 'sudo rm -fR base-path'
exec_on_all_nodes 'sudo rm  fclMainnet*.json'

# generate new Genesis Spec
scp reset/rebuild-spec.sh $node_00:/tmp/rebuild-spec.sh
ssh $node_00 "/tmp/rebuild-spec.sh $NODE_BUILD_ID"

# distrbute the specs, to the other nodes
ssh $node_00 'tar cz --exclude base-path fcl*.json' | ssh $node_01 'tar xz'
ssh $node_00 'tar cz --exclude base-path fcl*.json' | ssh $node_02 'tar xz'

# now change the compose files with the new node-build-id
subsitute_node_build_id $node_00 $NODE_BUILD_ID
subsitute_node_build_id $node_01 $NODE_BUILD_ID
subsitute_node_build_id $node_02 $NODE_BUILD_ID

# reboot $node_00 and find the bootnode id
# this by greping for the folowing and exiting the process on the pattern
# Local node identity is: (\w+)
ssh $node_00 'docker-compose up -d'
bootnode_id=$(ssh $node_00 "docker-compose logs | grep 'Local node identity is' | awk '{print \$10}'")

echo "Found bootnode_id $bootnode_id"

# change the docker-compose files splice in the new node-build-id for $node_01 and $node_02
# run the insert-keys step for all the nodes

# reboot them all
