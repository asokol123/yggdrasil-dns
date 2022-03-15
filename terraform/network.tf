resource "yandex_vpc_network" "network" {
  name = "yggdrasil-network"
}

resource "yandex_vpc_subnet" "subnet" {
  name           = "yggdrasil-subnet"
  zone           = "ru-central1-c"
  network_id     = yandex_vpc_network.network.id
  v4_cidr_blocks = ["10.128.0.0/24"]
}