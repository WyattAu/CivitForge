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
        "footer.powered_by" => "Powered by CivitForge".to_string(),
        // Keyboard Shortcuts
        "shortcuts.title" => "Keyboard Shortcuts".to_string(),
        "shortcuts.global" => "Global Navigation".to_string(),
        "shortcuts.repository" => "Repository".to_string(),
        "shortcuts.focus_search" => "Focus search bar".to_string(),
        "shortcuts.toggle_help" => "Toggle this help".to_string(),
        "shortcuts.go_home" => "Go to Home".to_string(),
        "shortcuts.go_repos" => "Go to Repositories".to_string(),
        "shortcuts.go_activity" => "Go to Activity".to_string(),
        "shortcuts.go_code" => "Go to Code".to_string(),
        "shortcuts.go_issues" => "Go to Issues".to_string(),
        "shortcuts.go_pulls" => "Go to Pull Requests".to_string(),
        "shortcuts.go_boards" => "Go to Boards".to_string(),
        // Profile
        "profile.title" => "User Profile".to_string(),
        "profile.display_name" => "Display Name".to_string(),
        "profile.bio" => "Bio".to_string(),
        "profile.avatar_url" => "Avatar URL".to_string(),
        "profile.location" => "Location".to_string(),
        "profile.website" => "Website".to_string(),
        "profile.update_success" => "Profile updated successfully.".to_string(),
        "profile.update_error" => "Failed to update profile.".to_string(),
        "profile.upload_avatar" => "Upload Avatar".to_string(),
        "profile.avatar_upload_success" => "Avatar uploaded successfully.".to_string(),
        "profile.avatar_upload_error" => "Failed to upload avatar.".to_string(),
        // Admin
        "admin.site_settings" => "Site Settings".to_string(),
        "admin.footer_text" => "Custom Footer Text".to_string(),
        "admin.logo_url" => "Logo URL".to_string(),
        "admin.save_settings" => "Save Settings".to_string(),
        "admin.settings_saved" => "Settings saved successfully.".to_string(),
        // Repo tabs
        "repo.tab.code" => "Code".to_string(),
        "repo.tab.issues" => "Issues".to_string(),
        "repo.tab.pulls" => "Pull Requests".to_string(),
        "repo.tab.boards" => "Boards".to_string(),
        "repo.tab.pipelines" => "Pipelines".to_string(),
        "repo.tab.wiki" => "Wiki".to_string(),
        "repo.tab.settings" => "Settings".to_string(),
        "repo.tab.commits" => "Commits".to_string(),
        "repo.tab.blame" => "Blame".to_string(),
        "repo.tab.releases" => "Releases".to_string(),
        // Settings sections
        "settings.general" => "General".to_string(),
        "settings.collaborators" => "Collaborators".to_string(),
        "settings.branches" => "Branches".to_string(),
        "settings.labels" => "Labels".to_string(),
        "settings.danger_zone" => "Danger Zone".to_string(),
        "settings.change_visibility" => "Change Visibility".to_string(),
        "settings.delete_repo" => "Delete this repository".to_string(),
        "settings.delete_repo_confirm" => "Once you delete a repository, there is no going back.".to_string(),
        "settings.ssh_keys" => "SSH Keys".to_string(),
        "settings.change_password" => "Change Password".to_string(),
        "settings.delete_account" => "Delete Account".to_string(),
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
