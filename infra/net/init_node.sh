#!/bin/bash

# Make sure we are root
  sudo su - root

# Install & Start nginx server
  apt-get install -y nginx
  systemctl start nginx
  systemctl enable nginx
  
