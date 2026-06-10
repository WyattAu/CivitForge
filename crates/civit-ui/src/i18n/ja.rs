pub fn get(key: &str) -> String {
    match key {
        // App
        "app.name" => "CivitForge".to_string(),
        // Nav
        "nav.home" => "ホーム".to_string(),
        "nav.repos" => "リポジトリ".to_string(),
        "nav.activity" => "アクティビティ".to_string(),
        "nav.explore" => "探索".to_string(),
        "nav.orgs" => "組織".to_string(),
        "nav.search" => "検索".to_string(),
        "nav.new_repo" => "新しいリポジトリ".to_string(),
        "nav.create" => "作成".to_string(),
        // Auth
        "auth.sign_in" => "ログイン".to_string(),
        "auth.sign_out" => "ログアウト".to_string(),
        "auth.register" => "登録".to_string(),
        "auth.username" => "ユーザー名".to_string(),
        "auth.email" => "メールアドレス".to_string(),
        "auth.password" => "パスワード".to_string(),
        "auth.display_name" => "表示名".to_string(),
        "auth.forgot_password" => "パスワードをお忘れですか？".to_string(),
        // Settings
        "settings.title" => "設定".to_string(),
        "settings.language" => "言語".to_string(),
        // Repo
        "repo.clone_url" => "クローンURL".to_string(),
        "repo.description" => "説明".to_string(),
        "repo.visibility" => "公開範囲".to_string(),
        "repo.public" => "公開".to_string(),
        "repo.private" => "非公開".to_string(),
        // Common
        "common.save" => "保存".to_string(),
        "common.cancel" => "キャンセル".to_string(),
        "common.delete" => "削除".to_string(),
        "common.edit" => "編集".to_string(),
        "common.search" => "検索".to_string(),
        "common.loading" => "読み込み中...".to_string(),
        "common.error" => "エラー".to_string(),
        "common.success" => "成功".to_string(),
        "common.back" => "戻る".to_string(),
        "common.next" => "次へ".to_string(),
        "common.previous" => "前へ".to_string(),
        "common.confirm" => "確認".to_string(),
        // Footer
        "footer.version" => "バージョン".to_string(),
        _ => key.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ja_translations() {
        assert_eq!(get("nav.home"), "ホーム");
        assert_eq!(get("auth.sign_in"), "ログイン");
        assert_eq!(get("common.save"), "保存");
    }
}
