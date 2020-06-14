provider "aws" {
  shared_credentials_file = "$HOME/.aws/credentials"
  profile                 = var.profile
  region                  = var.region
}

terraform {
  backend "s3" {
    bucket  = "pdf-generator-terraform"
    key     = "terraform.tfstate"
    region  = "us-east-2"
    encrypt = true
    profile = "earth"
  }
}

module "vpc" {
  source = "./modules/vpc"
}

module "ecr" {
  source       = "./modules/ecr"
  project_name = "${var.project_name}"
}

module "public_subnet" {
  source = "./modules/public-subnet"

  vpc_id = module.vpc.vpc_id
}

module "internet_gateway" {
  source = "./modules/internet-gateway"

  vpc_id = module.vpc.vpc_id
}

module "route_table" {
  source = "./modules/route-table"

  vpc_id              = module.vpc.vpc_id
  internet_gateway_id = module.internet_gateway.internet_gateway_id
  public_subnet_id    = module.public_subnet.public_subnet_id
}

module "ec2" {
  source = "./modules/ec2"

  vpc_id           = module.vpc.vpc_id
  public_subnet_id = module.public_subnet.public_subnet_id

  ec2_ssh_key_name        = var.ec2_ssh_key_name
  ec2_ssh_public_key_path = var.ec2_ssh_public_key_path
}
