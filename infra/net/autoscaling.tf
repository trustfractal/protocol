resource "aws_vpc" "fractal_protocol_vpc" {
  cidr_block       = "10.0.0.0/16"
  enable_dns_hostnames = true

  tags = {
    Name = "mainnet vpc"
  }
}

resource "aws_subnet" "public_eu_central_1a" {
  vpc_id     = aws_vpc.fractal_protocol_vpc.id
  cidr_block = "10.0.0.0/24"
  availability_zone = "eu-central-1a"

  tags = {
    Name = "Public Subnet eu-central-1a"
  }
}

resource "aws_subnet" "public_eu_central_1b" {
  vpc_id     = aws_vpc.fractal_protocol_vpc.id
  cidr_block = "10.0.1.0/24"
  availability_zone = "eu-central-1b"

  tags = {
    Name = "Public Subnet eu-central-1b"
  }
}

resource "aws_internet_gateway" "fractal_protocol_vpc_igw" {
  vpc_id = aws_vpc.fractal_protocol_vpc.id

  tags = {
    Name = "Fractal Protocol VPC - Internet Gateway"
  }
}

resource "aws_route_table" "fractal_protocol_vpc_public" {
    vpc_id = aws_vpc.fractal_protocol_vpc.id

    route {
        cidr_block = "0.0.0.0/0"
        gateway_id = aws_internet_gateway.fractal_protocol_vpc_igw.id
    }

    tags = {
        Name = "Public Subnets Route Table for Fractal Protocol VPC"
    }
}

resource "aws_route_table_association" "fractal_protocol_vpc_eu_central_1a_public" {
    subnet_id = aws_subnet.public_eu_central_1a.id
    route_table_id = aws_route_table.fractal_protocol_vpc_public.id
}

resource "aws_route_table_association" "fractal_protocol_vpc_eu_central_1b_public" {
    subnet_id = aws_subnet.public_eu_central_1b.id
    route_table_id = aws_route_table.fractal_protocol_vpc_public.id
}

resource "aws_security_group" "allow_https" {
  name        = "allow_https"
  description = "Allow HTTPS inbound connections"
  vpc_id = aws_vpc.fractal_protocol_vpc.id

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port       = 0
    to_port         = 0
    protocol        = "-1"
    cidr_blocks     = ["0.0.0.0/0"]
  }

  tags = {
    Name = "Allow HTTPS Security Group"
  }
}


resource "aws_security_group" "allow_nodefcl" {
  name        = "allow_nodefcl"
  description = "Allow nodefcl inbound connections"
  vpc_id = aws_vpc.fractal_protocol_vpc.id

  ingress {
    from_port   = 9944
    to_port     = 9944
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  ingress {
    from_port   = 9933
    to_port     = 9933
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 30333
    to_port     = 30333
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port       = 0
    to_port         = 0
    protocol        = "-1"
    cidr_blocks     = ["0.0.0.0/0"]
  }

  tags = {
    Name = "Allow NodeFcl Security Group"
  }
}

resource "aws_launch_configuration" "fcl_mainnet_node" {
  name_prefix = "fcl-mainnet-node"

  image_id = "ami-0dd02d7f96c7587ce" 
  instance_type = "t3.xlarge"
  # key_name = "Lenovo T410"

  security_groups = [ aws_security_group.allow_nodefcl.id ]
  associate_public_ip_address = true

  # rsync the snapshot from the boot node, and
  # start the compose
  user_data = <<USER_DATA
#!/bin/bash
#sudo -i -u ubuntu docker-compose down
#sudo -i -u ubuntu docker-compose up -d

sudo -i -u ubuntu 'echo "DONE" >user_data_worked'
  USER_DATA

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_security_group" "elb_https" {
  name        = "elb_https"
  description = "Allow HTTPS traffic to instances through Elastic Load Balancer"
  vpc_id = aws_vpc.fractal_protocol_vpc.id

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port       = 0
    to_port         = 0
    protocol        = "-1"
    cidr_blocks     = ["0.0.0.0/0"]
  }

  tags = {
    Name = "Allow HTTPS through ELB Security Group"
  }
}

resource "aws_acm_certificate" "fcl_mainnet_cert" {
  domain_name       = "nodes.mainnet.fractalprotocol.com"
  validation_method = "DNS"
}

resource "aws_elb" "nodes_elb" {
  name = "nodes-elb"
  security_groups = [
    aws_security_group.elb_https.id
  ]
  subnets = [
    aws_subnet.public_eu_central_1a.id,
    aws_subnet.public_eu_central_1b.id
  ]

  cross_zone_load_balancing   = true
  idle_timeout                = 3600
  connection_draining         = true
  connection_draining_timeout = 400

  health_check {
    healthy_threshold = 2
    unhealthy_threshold = 2
    timeout = 3
    interval = 30
    target = "HTTP:9933/health"
  }

  listener {
    lb_port = 443
    lb_protocol = "https"
    instance_port = 9944
    instance_protocol = "http"
    ssl_certificate_id = resource.aws_acm_certificate.fcl_mainnet_cert.arn
  }

}

resource "aws_lb_cookie_stickiness_policy" "nodes_elb_stickiness" {
  name                     = "nodes-elb-stickiness-policy"
  load_balancer            = aws_elb.nodes_elb.id
  lb_port                  = 443
  cookie_expiration_period = 3600
}

resource "aws_autoscaling_group" "fcl_mainnet_nodes" {
  name = "${aws_launch_configuration.fcl_mainnet_node.name}-asg"

  min_size             = 1
  desired_capacity     = 2
  max_size             = 8
  
  health_check_type    = "ELB"
  load_balancers = [
    aws_elb.nodes_elb.id
  ]

  launch_configuration = aws_launch_configuration.fcl_mainnet_node.name

  enabled_metrics = [
    "GroupMinSize",
    "GroupMaxSize",
    "GroupDesiredCapacity",
    "GroupInServiceInstances",
    "GroupTotalInstances"
  ]

  metrics_granularity = "1Minute"

  vpc_zone_identifier  = [
    aws_subnet.public_eu_central_1a.id,
    aws_subnet.public_eu_central_1b.id
  ]

  # Required to redeploy without an outage.
  lifecycle {
    create_before_destroy = true
  }

  tag {
    key                 = "Name"
    value               = "nodes"
    propagate_at_launch = true
  }

}

resource "aws_autoscaling_policy" "nodes_policy_up" {
  name = "nodes_policy_up"
  scaling_adjustment = 1
  adjustment_type = "ChangeInCapacity"
  cooldown = 300
  autoscaling_group_name = aws_autoscaling_group.fcl_mainnet_nodes.name
}

resource "aws_cloudwatch_metric_alarm" "nodes_cpu_alarm_up" {
  alarm_name = "nodes_cpu_alarm_up"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods = "2"
  metric_name = "CPUUtilization"
  namespace = "AWS/EC2"
  period = "120"
  statistic = "Average"
  threshold = "60"

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.fcl_mainnet_nodes.name
  }

  alarm_description = "This metric monitor EC2 instance CPU utilization"
  alarm_actions = [ aws_autoscaling_policy.nodes_policy_up.arn ]
}

resource "aws_autoscaling_policy" "nodes_policy_down" {
  name = "nodes_policy_down"
  scaling_adjustment = -1
  adjustment_type = "ChangeInCapacity"
  cooldown = 300
  autoscaling_group_name = aws_autoscaling_group.fcl_mainnet_nodes.name
}

resource "aws_cloudwatch_metric_alarm" "nodes_cpu_alarm_down" {
  alarm_name = "nodes_cpu_alarm_down"
  comparison_operator = "LessThanOrEqualToThreshold"
  evaluation_periods = "2"
  metric_name = "CPUUtilization"
  namespace = "AWS/EC2"
  period = "120"
  statistic = "Average"
  threshold = "10"

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.fcl_mainnet_nodes.name
  }

  alarm_description = "This metric monitor EC2 instance CPU utilization"
  alarm_actions = [ aws_autoscaling_policy.nodes_policy_down.arn ]
}

output "elb_dns_name" {
  value = aws_elb.nodes_elb.dns_name
}
