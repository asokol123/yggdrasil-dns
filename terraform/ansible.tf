resource "local_file" "ansible_inventory" {
  content = templatefile("inventory.tmpl", {
    vms = yandex_compute_instance.vm.*.network_interface.0.nat_ip_address
  })
  filename = "inventory"
}