use super::distro::get_distro;

pub fn get_os() -> String {
    get_distro().pretty_name
}
