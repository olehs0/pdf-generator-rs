variable "project_name" {
  type    = string
  default = "pdf-generator-rs"
}

variable "profile" {
  description = "AWS Profile"
  type        = string
  default     = "earth"
}

variable "region" {
  description = "Region for AWS resources"
  type        = string
  default     = "us-east-2"
}

variable "ec2_ssh_key_name" {
  description = "The SSH Key Name"
  type        = string
  default     = "pdf-generator-ec2-key"
}

variable "ec2_ssh_public_key_path" {
  description = "The local path to the SSH Public Key"
  type        = string
  default     = "~/.ssh/id_rsa.pub"
}
