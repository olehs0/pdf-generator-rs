resource "aws_ecr_repository" "pdf_generator_dockers" {
  name                 = "pdf-generator"
  image_tag_mutability = "MUTABLE"
}

output "pdf_gen_repo_url" {
  value = "${aws_ecr_repository.pdf_generator_dockers.repository_url}"
}
