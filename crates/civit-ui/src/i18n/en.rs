pub fn get(key: &str) -> String {
    match key {
        // App
        "app.name" => "CivitForge".to_string(),
        // Nav
        "nav.home" => "Home".to_string(),
        "nav.repos" => "Repositories".to_string(),
        "nav.activity" => "Activity".to_string(),
        "nav.explore" => "Explore".to_string(),
        "nav.orgs" => "Organizations".to_string(),
        "nav.search" => "Search".to_string(),
        "nav.new_repo" => "New Repo".to_string(),
        "nav.create" => "Create".to_string(),
        // Auth
        "auth.sign_in" => "Sign In".to_string(),
        "auth.sign_out" => "Sign Out".to_string(),
        "auth.register" => "Register".to_string(),
        "auth.username" => "Username".to_string(),
        "auth.email" => "Email".to_string(),
        "auth.password" => "Password".to_string(),
        "auth.display_name" => "Display Name".to_string(),
        "auth.forgot_password" => "Forgot Password?".to_string(),
        // Settings
        "settings.title" => "Settings".to_string(),
        "settings.language" => "Language".to_string(),
        // Repo
        "repo.clone_url" => "Clone URL".to_string(),
        "repo.description" => "Description".to_string(),
        "repo.visibility" => "Visibility".to_string(),
        "repo.public" => "Public".to_string(),
        "repo.private" => "Private".to_string(),
        // Common
        "common.save" => "Save".to_string(),
        "common.cancel" => "Cancel".to_string(),
        "common.delete" => "Delete".to_string(),
        "common.edit" => "Edit".to_string(),
        "common.search" => "Search".to_string(),
        "common.loading" => "Loading...".to_string(),
        "common.error" => "Error".to_string(),
        "common.success" => "Success".to_string(),
        "common.back" => "Back".to_string(),
        "common.next" => "Next".to_string(),
        "common.previous" => "Previous".to_string(),
        "common.confirm" => "Confirm".to_string(),
        // Footer
        "footer.version" => "Version".to_string(),
        _ => key.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_en_translations() {
        assert_eq!(get("app.name"), "CivitForge");
        assert_eq!(get("nav.home"), "Home");
        assert_eq!(get("auth.sign_in"), "Sign In");
        assert_eq!(get("nonexistent"), "nonexistent");
    }
}
