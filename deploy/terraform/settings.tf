terraform {
  required_providers {
    yandex = {
      source  = "yandex-cloud/yandex"
      version = "0.69.0"
    }
  }
}

variable "yc_oauth" {
  type        = string
  description = "yandex cloud oauth token"
}

locals {
  folder_id = "b1g3rlimsm56rt5r8jo4"
  image_id  = "fd83n3uou8m03iq9gavu"
}

provider "yandex" {
  token     = var.yc_oauth
  cloud_id  = "b1g0afo4omio7j1jbsu9"
  folder_id = local.folder_id
  zone      = "ru-central1-c"
}