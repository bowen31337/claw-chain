use substrate_build_script_utils::rerun_if_git_head_changed;

fn main() {
    rerun_if_git_head_changed();
    substrate_build_script_utils::generate_cargo_keys();
}
