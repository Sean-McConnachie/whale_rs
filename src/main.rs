fn main() {
    let mut program_state = {
        let config = whale_rs::config::read_or_create_all_configs();
        if !config.core.data_dir.exists() {
            std::fs::create_dir_all(&config.core.data_dir).unwrap();
        }

        let current_working_directory = std::env::current_dir().unwrap();

        whale_rs::state::ProgramState::init(config, current_working_directory)
    };
}
