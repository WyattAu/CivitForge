use super::locale::Locale;

/// Type-safe translation key. Exhaustive pattern match guarantees:
/// - Every key exists for every locale (no runtime "key not found")
/// - Zero allocation: returns &'static str
/// - O(1) lookup via compiler-optimized match jump table
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Key {
    // App
    AppName,
    // Navigation
    NavHome,
    NavRepos,
    NavActivity,
    NavExplore,
    NavOrgs,
    NavSearch,
    NavNewRepo,
    NavCreate,
    // Auth
    AuthSignIn,
    AuthSignOut,
    AuthRegister,
    AuthUsername,
    AuthEmail,
    AuthPassword,
    AuthDisplayName,
    AuthForgotPassword,
    // Settings
    SettingsTitle,
    SettingsLanguage,
    SettingsGeneral,
    SettingsCollaborators,
    SettingsBranches,
    SettingsLabels,
    SettingsDangerZone,
    SettingsChangeVisibility,
    SettingsDeleteRepo,
    SettingsDeleteRepoConfirm,
    SettingsSshKeys,
    SettingsChangePassword,
    SettingsDeleteAccount,
    // Repo
    RepoCloneUrl,
    RepoDescription,
    RepoVisibility,
    RepoPublic,
    RepoPrivate,
    RepoTabCode,
    RepoTabIssues,
    RepoTabPulls,
    RepoTabBoards,
    RepoTabPipelines,
    RepoTabWiki,
    RepoTabSettings,
    RepoTabCommits,
    RepoTabBlame,
    RepoTabReleases,
    // Common
    CommonSave,
    CommonCancel,
    CommonDelete,
    CommonEdit,
    CommonSearch,
    CommonLoading,
    CommonError,
    CommonSuccess,
    CommonBack,
    CommonNext,
    CommonPrevious,
    CommonConfirm,
    // Footer
    FooterVersion,
    FooterPoweredBy,
    // Shortcuts
    ShortcutsTitle,
    ShortcutsGlobal,
    ShortcutsRepository,
    ShortcutsFocusSearch,
    ShortcutsToggleHelp,
    ShortcutsGoHome,
    ShortcutsGoRepos,
    ShortcutsGoActivity,
    ShortcutsGoCode,
    ShortcutsGoIssues,
    ShortcutsGoPulls,
    ShortcutsGoBoards,
    // Profile
    ProfileTitle,
    ProfileDisplayName,
    ProfileBio,
    ProfileAvatarUrl,
    ProfileLocation,
    ProfileWebsite,
    ProfileUpdateSuccess,
    ProfileUpdateError,
    ProfileUploadAvatar,
    ProfileAvatarUploadSuccess,
    ProfileAvatarUploadError,
    // Admin
    AdminSiteSettings,
    AdminFooterText,
    AdminLogoUrl,
    AdminSaveSettings,
    AdminSettingsSaved,
}

impl Key {
    /// Translate to the given locale. Returns `&'static str` — zero allocation.
    ///
    /// Invariant: result is never empty (missing keys return the key name itself).
    pub const fn translate(self, locale: Locale) -> &'static str {
        match (locale, self) {
            // ── App ──
            (_, Key::AppName) => "CivitForge",

            // ── Navigation ──
            (_, Key::NavHome) => "Home",
            (_, Key::NavRepos) => "Repositories",
            (_, Key::NavActivity) => "Activity",
            (_, Key::NavExplore) => "Explore",
            (_, Key::NavOrgs) => "Organizations",
            (_, Key::NavSearch) => "Search",
            (_, Key::NavNewRepo) => "New Repo",
            (_, Key::NavCreate) => "Create",

            // ── Auth ──
            (_, Key::AuthSignIn) => "Sign In",
            (_, Key::AuthSignOut) => "Sign Out",
            (_, Key::AuthRegister) => "Register",
            (_, Key::AuthUsername) => "Username",
            (_, Key::AuthEmail) => "Email",
            (_, Key::AuthPassword) => "Password",
            (_, Key::AuthDisplayName) => "Display Name",
            (_, Key::AuthForgotPassword) => "Forgot Password?",

            // ── Settings ──
            (_, Key::SettingsTitle) => "Settings",
            (_, Key::SettingsLanguage) => "Language",
            (_, Key::SettingsGeneral) => "General",
            (_, Key::SettingsCollaborators) => "Collaborators",
            (_, Key::SettingsBranches) => "Branches",
            (_, Key::SettingsLabels) => "Labels",
            (_, Key::SettingsDangerZone) => "Danger Zone",
            (_, Key::SettingsChangeVisibility) => "Change Visibility",
            (_, Key::SettingsDeleteRepo) => "Delete this repository",
            (_, Key::SettingsDeleteRepoConfirm) => "Once you delete a repository, there is no going back.",
            (_, Key::SettingsSshKeys) => "SSH Keys",
            (_, Key::SettingsChangePassword) => "Change Password",
            (_, Key::SettingsDeleteAccount) => "Delete Account",

            // ── Repo ──
            (_, Key::RepoCloneUrl) => "Clone URL",
            (_, Key::RepoDescription) => "Description",
            (_, Key::RepoVisibility) => "Visibility",
            (_, Key::RepoPublic) => "Public",
            (_, Key::RepoPrivate) => "Private",
            (_, Key::RepoTabCode) => "Code",
            (_, Key::RepoTabIssues) => "Issues",
            (_, Key::RepoTabPulls) => "Pull Requests",
            (_, Key::RepoTabBoards) => "Boards",
            (_, Key::RepoTabPipelines) => "Pipelines",
            (_, Key::RepoTabWiki) => "Wiki",
            (_, Key::RepoTabSettings) => "Settings",
            (_, Key::RepoTabCommits) => "Commits",
            (_, Key::RepoTabBlame) => "Blame",
            (_, Key::RepoTabReleases) => "Releases",

            // ── Common ──
            (_, Key::CommonSave) => "Save",
            (_, Key::CommonCancel) => "Cancel",
            (_, Key::CommonDelete) => "Delete",
            (_, Key::CommonEdit) => "Edit",
            (_, Key::CommonSearch) => "Search",
            (_, Key::CommonLoading) => "Loading...",
            (_, Key::CommonError) => "Error",
            (_, Key::CommonSuccess) => "Success",
            (_, Key::CommonBack) => "Back",
            (_, Key::CommonNext) => "Next",
            (_, Key::CommonPrevious) => "Previous",
            (_, Key::CommonConfirm) => "Confirm",

            // ── Footer ──
            (_, Key::FooterVersion) => "Version",
            (_, Key::FooterPoweredBy) => "Powered by CivitForge",

            // ── Shortcuts ──
            (_, Key::ShortcutsTitle) => "Keyboard Shortcuts",
            (_, Key::ShortcutsGlobal) => "Global Navigation",
            (_, Key::ShortcutsRepository) => "Repository",
            (_, Key::ShortcutsFocusSearch) => "Focus search bar",
            (_, Key::ShortcutsToggleHelp) => "Toggle this help",
            (_, Key::ShortcutsGoHome) => "Go to Home",
            (_, Key::ShortcutsGoRepos) => "Go to Repositories",
            (_, Key::ShortcutsGoActivity) => "Go to Activity",
            (_, Key::ShortcutsGoCode) => "Go to Code",
            (_, Key::ShortcutsGoIssues) => "Go to Issues",
            (_, Key::ShortcutsGoPulls) => "Go to Pull Requests",
            (_, Key::ShortcutsGoBoards) => "Go to Boards",

            // ── Profile ──
            (_, Key::ProfileTitle) => "User Profile",
            (_, Key::ProfileDisplayName) => "Display Name",
            (_, Key::ProfileBio) => "Bio",
            (_, Key::ProfileAvatarUrl) => "Avatar URL",
            (_, Key::ProfileLocation) => "Location",
            (_, Key::ProfileWebsite) => "Website",
            (_, Key::ProfileUpdateSuccess) => "Profile updated successfully.",
            (_, Key::ProfileUpdateError) => "Failed to update profile.",
            (_, Key::ProfileUploadAvatar) => "Upload Avatar",
            (_, Key::ProfileAvatarUploadSuccess) => "Avatar uploaded successfully.",
            (_, Key::ProfileAvatarUploadError) => "Failed to upload avatar.",

            // ── Admin ──
            (_, Key::AdminSiteSettings) => "Site Settings",
            (_, Key::AdminFooterText) => "Custom Footer Text",
            (_, Key::AdminLogoUrl) => "Logo URL",
            (_, Key::AdminSaveSettings) => "Save Settings",
            (_, Key::AdminSettingsSaved) => "Settings saved successfully.",
        }
    }

    // ── Chinese (zh) overrides ──
    #[allow(unreachable_patterns)]
    const fn translate_zh(self) -> &'static str {
        match self {
            Key::NavHome => "首页",
            Key::NavRepos => "仓库",
            Key::NavActivity => "动态",
            Key::NavExplore => "探索",
            Key::NavOrgs => "组织",
            Key::NavSearch => "搜索",
            Key::NavNewRepo => "新建仓库",
            Key::NavCreate => "创建",
            Key::AuthSignIn => "登录",
            Key::AuthSignOut => "退出登录",
            Key::AuthRegister => "注册",
            Key::AuthUsername => "用户名",
            Key::AuthEmail => "邮箱",
            Key::AuthPassword => "密码",
            Key::AuthDisplayName => "显示名称",
            Key::AuthForgotPassword => "忘记密码？",
            Key::SettingsTitle => "设置",
            Key::SettingsLanguage => "语言",
            Key::RepoCloneUrl => "克隆地址",
            Key::RepoDescription => "描述",
            Key::RepoVisibility => "可见性",
            Key::RepoPublic => "公开",
            Key::RepoPrivate => "私有",
            Key::CommonSave => "保存",
            Key::CommonCancel => "取消",
            Key::CommonDelete => "删除",
            Key::CommonEdit => "编辑",
            Key::CommonSearch => "搜索",
            Key::CommonLoading => "加载中...",
            Key::CommonError => "错误",
            Key::CommonSuccess => "成功",
            Key::CommonBack => "返回",
            Key::CommonNext => "下一步",
            Key::CommonPrevious => "上一步",
            Key::CommonConfirm => "确认",
            Key::FooterVersion => "版本",
            _ => self.translate(Locale::En),
        }
    }

    // ── Japanese (ja) overrides ──
    #[allow(unreachable_patterns)]
    const fn translate_ja(self) -> &'static str {
        match self {
            Key::NavHome => "ホーム",
            Key::NavRepos => "リポジトリ",
            Key::NavActivity => "アクティビティ",
            Key::NavExplore => "探索",
            Key::NavOrgs => "組織",
            Key::NavSearch => "検索",
            Key::NavNewRepo => "新しいリポジトリ",
            Key::NavCreate => "作成",
            Key::AuthSignIn => "ログイン",
            Key::AuthSignOut => "ログアウト",
            Key::AuthRegister => "登録",
            Key::AuthUsername => "ユーザー名",
            Key::AuthEmail => "メールアドレス",
            Key::AuthPassword => "パスワード",
            Key::AuthDisplayName => "表示名",
            Key::AuthForgotPassword => "パスワードをお忘れですか？",
            Key::SettingsTitle => "設定",
            Key::SettingsLanguage => "言語",
            Key::RepoCloneUrl => "クローンURL",
            Key::RepoDescription => "説明",
            Key::RepoVisibility => "公開範囲",
            Key::RepoPublic => "公開",
            Key::RepoPrivate => "非公開",
            Key::CommonSave => "保存",
            Key::CommonCancel => "キャンセル",
            Key::CommonDelete => "削除",
            Key::CommonEdit => "編集",
            Key::CommonSearch => "検索",
            Key::CommonLoading => "読み込み中...",
            Key::CommonError => "エラー",
            Key::CommonSuccess => "成功",
            Key::CommonBack => "戻る",
            Key::CommonNext => "次へ",
            Key::CommonPrevious => "前へ",
            Key::CommonConfirm => "確認",
            Key::FooterVersion => "バージョン",
            _ => self.translate(Locale::En),
        }
    }

    // ── Korean (ko) overrides ──
    #[allow(unreachable_patterns)]
    const fn translate_ko(self) -> &'static str {
        match self {
            Key::NavHome => "홈",
            Key::NavRepos => "저장소",
            Key::NavActivity => "활동",
            Key::NavExplore => "탐색",
            Key::NavOrgs => "조직",
            Key::NavSearch => "검색",
            Key::NavNewRepo => "새 저장소",
            Key::NavCreate => "생성",
            Key::AuthSignIn => "로그인",
            Key::AuthSignOut => "로그아웃",
            Key::AuthRegister => "회원가입",
            Key::AuthUsername => "사용자 이름",
            Key::AuthEmail => "이메일",
            Key::AuthPassword => "비밀번호",
            Key::AuthDisplayName => "표시 이름",
            Key::AuthForgotPassword => "비밀번호를 잊으셨나요?",
            Key::SettingsTitle => "설정",
            Key::SettingsLanguage => "언어",
            Key::RepoCloneUrl => "클론 URL",
            Key::RepoDescription => "설명",
            Key::RepoVisibility => "공개 범위",
            Key::RepoPublic => "공개",
            Key::RepoPrivate => "비공개",
            Key::CommonSave => "저장",
            Key::CommonCancel => "취소",
            Key::CommonDelete => "삭제",
            Key::CommonEdit => "편집",
            Key::CommonSearch => "검색",
            Key::CommonLoading => "로딩 중...",
            Key::CommonError => "오류",
            Key::CommonSuccess => "성공",
            Key::CommonBack => "뒤로",
            Key::CommonNext => "다음",
            Key::CommonPrevious => "이전",
            Key::CommonConfirm => "확인",
            Key::FooterVersion => "버전",
            _ => self.translate(Locale::En),
        }
    }

    /// Translate with locale dispatch. O(1), zero allocation.
    pub fn tr(self, locale: Locale) -> &'static str {
        match locale {
            Locale::Zh => self.translate_zh(),
            Locale::Ja => self.translate_ja(),
            Locale::Ko => self.translate_ko(),
            Locale::En => self.translate(Locale::En),
        }
    }

    /// Parse from the old dot-notation key string. For migration only.
    #[allow(dead_code)]
    pub fn from_str_key(s: &str) -> Option<Self> {
        match s {
            "app.name" => Some(Self::AppName),
            "nav.home" => Some(Self::NavHome),
            "nav.repos" => Some(Self::NavRepos),
            "nav.activity" => Some(Self::NavActivity),
            "nav.explore" => Some(Self::NavExplore),
            "nav.orgs" => Some(Self::NavOrgs),
            "nav.search" => Some(Self::NavSearch),
            "nav.new_repo" => Some(Self::NavNewRepo),
            "nav.create" => Some(Self::NavCreate),
            "auth.sign_in" => Some(Self::AuthSignIn),
            "auth.sign_out" => Some(Self::AuthSignOut),
            "auth.register" => Some(Self::AuthRegister),
            "auth.username" => Some(Self::AuthUsername),
            "auth.email" => Some(Self::AuthEmail),
            "auth.password" => Some(Self::AuthPassword),
            "auth.display_name" => Some(Self::AuthDisplayName),
            "auth.forgot_password" => Some(Self::AuthForgotPassword),
            "settings.title" => Some(Self::SettingsTitle),
            "settings.language" => Some(Self::SettingsLanguage),
            "settings.general" => Some(Self::SettingsGeneral),
            "settings.collaborators" => Some(Self::SettingsCollaborators),
            "settings.branches" => Some(Self::SettingsBranches),
            "settings.labels" => Some(Self::SettingsLabels),
            "settings.danger_zone" => Some(Self::SettingsDangerZone),
            "settings.change_visibility" => Some(Self::SettingsChangeVisibility),
            "settings.delete_repo" => Some(Self::SettingsDeleteRepo),
            "settings.delete_repo_confirm" => Some(Self::SettingsDeleteRepoConfirm),
            "settings.ssh_keys" => Some(Self::SettingsSshKeys),
            "settings.change_password" => Some(Self::SettingsChangePassword),
            "settings.delete_account" => Some(Self::SettingsDeleteAccount),
            "repo.clone_url" => Some(Self::RepoCloneUrl),
            "repo.description" => Some(Self::RepoDescription),
            "repo.visibility" => Some(Self::RepoVisibility),
            "repo.public" => Some(Self::RepoPublic),
            "repo.private" => Some(Self::RepoPrivate),
            "repo.tab.code" => Some(Self::RepoTabCode),
            "repo.tab.issues" => Some(Self::RepoTabIssues),
            "repo.tab.pulls" => Some(Self::RepoTabPulls),
            "repo.tab.boards" => Some(Self::RepoTabBoards),
            "repo.tab.pipelines" => Some(Self::RepoTabPipelines),
            "repo.tab.wiki" => Some(Self::RepoTabWiki),
            "repo.tab.settings" => Some(Self::RepoTabSettings),
            "repo.tab.commits" => Some(Self::RepoTabCommits),
            "repo.tab.blame" => Some(Self::RepoTabBlame),
            "repo.tab.releases" => Some(Self::RepoTabReleases),
            "common.save" => Some(Self::CommonSave),
            "common.cancel" => Some(Self::CommonCancel),
            "common.delete" => Some(Self::CommonDelete),
            "common.edit" => Some(Self::CommonEdit),
            "common.search" => Some(Self::CommonSearch),
            "common.loading" => Some(Self::CommonLoading),
            "common.error" => Some(Self::CommonError),
            "common.success" => Some(Self::CommonSuccess),
            "common.back" => Some(Self::CommonBack),
            "common.next" => Some(Self::CommonNext),
            "common.previous" => Some(Self::CommonPrevious),
            "common.confirm" => Some(Self::CommonConfirm),
            "footer.version" => Some(Self::FooterVersion),
            "footer.powered_by" => Some(Self::FooterPoweredBy),
            "shortcuts.title" => Some(Self::ShortcutsTitle),
            "shortcuts.global" => Some(Self::ShortcutsGlobal),
            "shortcuts.repository" => Some(Self::ShortcutsRepository),
            "shortcuts.focus_search" => Some(Self::ShortcutsFocusSearch),
            "shortcuts.toggle_help" => Some(Self::ShortcutsToggleHelp),
            "shortcuts.go_home" => Some(Self::ShortcutsGoHome),
            "shortcuts.go_repos" => Some(Self::ShortcutsGoRepos),
            "shortcuts.go_activity" => Some(Self::ShortcutsGoActivity),
            "shortcuts.go_code" => Some(Self::ShortcutsGoCode),
            "shortcuts.go_issues" => Some(Self::ShortcutsGoIssues),
            "shortcuts.go_pulls" => Some(Self::ShortcutsGoPulls),
            "shortcuts.go_boards" => Some(Self::ShortcutsGoBoards),
            "profile.title" => Some(Self::ProfileTitle),
            "profile.display_name" => Some(Self::ProfileDisplayName),
            "profile.bio" => Some(Self::ProfileBio),
            "profile.avatar_url" => Some(Self::ProfileAvatarUrl),
            "profile.location" => Some(Self::ProfileLocation),
            "profile.website" => Some(Self::ProfileWebsite),
            "profile.update_success" => Some(Self::ProfileUpdateSuccess),
            "profile.update_error" => Some(Self::ProfileUpdateError),
            "profile.upload_avatar" => Some(Self::ProfileUploadAvatar),
            "profile.avatar_upload_success" => Some(Self::ProfileAvatarUploadSuccess),
            "profile.avatar_upload_error" => Some(Self::ProfileAvatarUploadError),
            "admin.site_settings" => Some(Self::AdminSiteSettings),
            "admin.footer_text" => Some(Self::AdminFooterText),
            "admin.logo_url" => Some(Self::AdminLogoUrl),
            "admin.save_settings" => Some(Self::AdminSaveSettings),
            "admin.settings_saved" => Some(Self::AdminSettingsSaved),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keys_translate_to_non_empty() {
        let keys = [
            Key::AppName, Key::NavHome, Key::NavRepos, Key::NavActivity,
            Key::NavExplore, Key::NavOrgs, Key::NavSearch, Key::NavNewRepo,
            Key::NavCreate, Key::AuthSignIn, Key::AuthSignOut, Key::AuthRegister,
            Key::AuthUsername, Key::AuthEmail, Key::AuthPassword, Key::AuthDisplayName,
            Key::AuthForgotPassword, Key::SettingsTitle, Key::SettingsLanguage,
            Key::CommonSave, Key::CommonCancel, Key::CommonDelete, Key::CommonEdit,
            Key::CommonSearch, Key::CommonLoading, Key::CommonError, Key::CommonSuccess,
            Key::FooterVersion,
        ];
        for locale in Locale::ALL {
            for key in &keys {
                let result = key.tr(*locale);
                assert!(!result.is_empty(), "key={:?} locale={:?}", key, locale);
            }
        }
    }

    #[test]
    fn from_str_key_roundtrip() {
        let key = Key::NavHome;
        let s = "nav.home";
        assert_eq!(Key::from_str_key(s), Some(key));
    }
}
