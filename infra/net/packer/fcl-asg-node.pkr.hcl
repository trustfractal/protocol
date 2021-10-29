packer {
  required_plugins {
    amazon = {
      version = ">= 0.0.2"
      source  = "github.com/hashicorp/amazon"
    }
  }
}

source "amazon-ebs" "ubuntu" {
  ami_name      = "fcl-asg-node-ubuntu-focal"
  instance_type = "t3.xlarge"
  region        = "eu-central-1"
  source_ami_filter {
    filters = {
      name                = "ubuntu/images/*ubuntu-focal-20.04-amd64-server-*"
      root-device-type    = "ebs"
      virtualization-type = "hvm"
    }
    most_recent = true
    owners      = ["099720109477"]
  }
  launch_block_device_mappings {
    device_name = "/dev/sda1"
    volume_size = 512
    delete_on_termination = true
  }
  ssh_username = "ubuntu"
}

build {
  name = "fcl-asg-node"
  sources = [
    "source.amazon-ebs.ubuntu"
  ]

  provisioner "file" {
    sources = ["../files/asg/docker-compose.yml", "../files/spec/fclMainnetSpecRaw.json"]
    destination = "/home/ubuntu/"
  }

  provisioner "shell" {
    script = "fcl-asg-node/provision.sh"
  }
}
