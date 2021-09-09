# Configure the AWS Provider
provider "aws" {
   region  = var.region
}

# Create VPC
resource "aws_vpc" "main" {
  cidr_block = var.vpc_cidr

  tags = {
    Project = "fclnet-provisioning"
    Name = "FclNet VPC"
 }
}

# Create Public Subnet1
resource "aws_subnet" "pub_sub1" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.pub_sub1_cidr_block
  availability_zone       = "${var.region}a"
  map_public_ip_on_launch = true
  tags = {
    Project = "fclnet-provisioning"
     Name = "public_subnet1"
 
 }
}

# Create Public Subnet2
resource "aws_subnet" "pub_sub2" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.pub_sub2_cidr_block
  availability_zone       = "${var.region}b"
  map_public_ip_on_launch = true
  tags = {
    Project = "fclnet-provisioning"
    Name = "public_subnet2" 
 }
}

# Create Private Subnet1
resource "aws_subnet" "prv_sub1" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.prv_sub1_cidr_block
  availability_zone       = "${var.region}a"
  map_public_ip_on_launch = false

  tags = {
    Project = "fclnet-provisioning"
    Name = "private_subnet1" 
 }
}

# Create Private Subnet2
resource "aws_subnet" "prv_sub2" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.prv_sub2_cidr_block
  availability_zone       = "${var.region}b"
  map_public_ip_on_launch = false

  tags = {
    Project = "fclnet-provisioning"
    Name = "private_subnet2"
  }
}

# Create Internet Gateway
resource "aws_internet_gateway" "igw" {
  vpc_id = aws_vpc.main.id

  tags = {
    Project = "fclnet-provisioning"
    Name = "internet gateway" 
 }
}

# Create Public Route Table
resource "aws_route_table" "pub_sub1_rt" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.igw.id
  }

  tags = {
    Project = "fclnet-provisioning"
    Name = "public subnet route table" 
 }
}

# Create route table association of public subnet1
resource "aws_route_table_association" "internet_for_pub_sub1" {
  route_table_id = aws_route_table.pub_sub1_rt.id
  subnet_id      = aws_subnet.pub_sub1.id
}

# Create route table association of public subnet2
resource "aws_route_table_association" "internet_for_pub_sub2" {
  route_table_id = aws_route_table.pub_sub1_rt.id
  subnet_id      = aws_subnet.pub_sub2.id
}

# Create EIP for NAT GW1
resource "aws_eip" "eip_natgw1" {
  count = "1"
}

# Create NAT gateway1
resource "aws_nat_gateway" "natgateway_1" {
  count         = "1"
  allocation_id = aws_eip.eip_natgw1[count.index].id
  subnet_id     = aws_subnet.pub_sub1.id
}

# Create EIP for NAT GW2
resource "aws_eip" "eip_natgw2" {
  count = "1"
}

# Create NAT gateway2
resource "aws_nat_gateway" "natgateway_2" {
  count         = "1"
  allocation_id = aws_eip.eip_natgw2[count.index].id
  subnet_id     = aws_subnet.pub_sub2.id
}

# Create private route table for prv sub1
resource "aws_route_table" "prv_sub1_rt" {
  count  = "1"
  vpc_id = aws_vpc.main.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.natgateway_1[count.index].id
  }
  tags = {
    Project = "fclnet-provisioning"
    Name = "private subnet1 route table" 
 }
}

# Create route table association betn prv sub1 & NAT GW1
resource "aws_route_table_association" "pri_sub1_to_natgw1" {
  count          = "1"
  route_table_id = aws_route_table.prv_sub1_rt[count.index].id
  subnet_id      = aws_subnet.prv_sub1.id
}

# Create private route table for prv sub2
resource "aws_route_table" "prv_sub2_rt" {
  count  = "1"
  vpc_id = aws_vpc.main.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.natgateway_2[count.index].id
  }
  tags = {
    Project = "fclnet-provisioning"
    Name = "private subnet2 route table"
  }
}

# Create route table association betn prv sub2 & NAT GW2
resource "aws_route_table_association" "pri_sub2_to_natgw1" {
  count          = "1"
  route_table_id = aws_route_table.prv_sub2_rt[count.index].id
  subnet_id      = aws_subnet.prv_sub2.id
}


# Create security group for load balancer
resource "aws_security_group" "elb_sg" {
  name        = var.sg_name
  description = var.sg_description
  vpc_id      = aws_vpc.main.id

ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    description = "HTTP"
    cidr_blocks = ["0.0.0.0/0"]
  }

egress {
    from_port        = 0
    to_port          = 0
    protocol         = "-1"
    cidr_blocks      = ["0.0.0.0/0"]
    ipv6_cidr_blocks = ["::/0"]
  }
 
 tags = {
    Name = var.sg_tagname
    Project = "fclnet-provisioning" 
  }	
}

# Create security group for nodefcl

resource "aws_security_group" "fclnode" {
  name        = var.sg_ws_name
  description = var.sg_ws_description
  vpc_id      = aws_vpc.main.id

  ingress {
      from_port   = 80
      to_port     = 80
      protocol    = "tcp"
      description = "HTTP"
      cidr_blocks = ["0.0.0.0/0"]

   }

  ingress {
      from_port   = 22
      to_port     = 22
      protocol    = "tcp"
      description = "ssh"
      cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
      from_port        = 0
      to_port          = 0
      protocol         = "-1"
      cidr_blocks      = ["0.0.0.0/0"]
      ipv6_cidr_blocks = ["::/0"]
  }

  tags = {
      Name = var.sg_ws_tagname 
      Project = "fclnet-provisioning"
  }
}

# Create Launch config
resource "aws_launch_configuration" "nodefcl-launch-config" {
  name_prefix   = "nodefcl-launch-config"
  image_id      =  var.ami
  instance_type = "t3.xlarge"
  key_name	= var.keyname
  security_groups = ["${aws_security_group.fclnode.id}"]

  associate_public_ip_address = true
  
  lifecycle {
    create_before_destroy = true
  }

  user_data = filebase64("${path.module}/init_node.sh")
}

# Create Auto Scaling Group
resource "aws_autoscaling_group" "FclNet-ASG-tf" {
  name		             = "FclNet-ASG-tf"
  desired_capacity     = 3
  max_size             = 10
  min_size             = 1
  force_delete         = true
  depends_on 	         = [aws_lb.ALB-tf]
  target_group_arns    = ["${aws_lb_target_group.TG-tf.arn}"]
  health_check_type    = "EC2"
  launch_configuration = aws_launch_configuration.nodefcl-launch-config.name

  # change pub_sub{id} in prv_sub{id} to shield nodes from public traffic
  vpc_zone_identifier = ["${aws_subnet.pub_sub1.id}","${aws_subnet.pub_sub2.id}"]
  
  tag {
    key                 = "Name"
    value               = "FclNet-ASG-tf"
    propagate_at_launch = true
  }
} 

# Create Target group
resource "aws_lb_target_group" "TG-tf" {
  name     = "FclNet-TargetGroup-tf"
  depends_on = [aws_vpc.main]
  port     = 80
  protocol = "HTTP"
  vpc_id   = "${aws_vpc.main.id}"
  health_check {
    interval            = 70
    path                = "/index.html"
    port                = 80
    healthy_threshold   = 2
    unhealthy_threshold = 2
    timeout             = 60 
    protocol            = "HTTP"
    matcher             = "200,202"
  }
}

# Create ALB
resource "aws_lb" "ALB-tf" {
   name              = "FclNet-ALG-tf"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.elb_sg.id]
  subnets            = [aws_subnet.pub_sub1.id,aws_subnet.pub_sub2.id]

  tags = {
      name  = "FclNet-AppLoadBalancer-tf"
    	Project = "fclnet-provisioning"
  }
}

# Create ALB Listener 
resource "aws_lb_listener" "front_end" {
  load_balancer_arn = aws_lb.ALB-tf.arn
  port              = "80"
  protocol          = "HTTP"
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.TG-tf.arn
  }
}
