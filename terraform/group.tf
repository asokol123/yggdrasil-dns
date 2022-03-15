resource "yandex_compute_instance" "vm" {
  name        = "hse-yggdrasil-${count.index}"
  count       = 7
  platform_id = "standard-v3"
  resources {
    memory = 2
    cores  = 2
  }
  boot_disk {
    initialize_params {
      image_id = local.image_id
      size     = 15
    }
  }
  network_interface {
    subnet_id = yandex_vpc_subnet.subnet.id
    nat       = true
  }
  metadata = {
    ssh-keys = "ubuntu:${file("~/.ssh/id_rsa.pub")}"
  }
}