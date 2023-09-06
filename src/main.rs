
fn main() {
    let config = whale_rs::config::read_or_create_all_configs();
    if !config.core.data_dir.exists() {
        std::fs::create_dir_all(&config.core.data_dir).unwrap();
    }

}
