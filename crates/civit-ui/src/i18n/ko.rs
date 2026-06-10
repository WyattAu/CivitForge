pub fn get(key: &str) -> String {
    match key {
        // App
        "app.name" => "CivitForge".to_string(),
        // Nav
        "nav.home" => "홈".to_string(),
        "nav.repos" => "저장소".to_string(),
        "nav.activity" => "활동".to_string(),
        "nav.explore" => "탐색".to_string(),
        "nav.orgs" => "조직".to_string(),
        "nav.search" => "검색".to_string(),
        "nav.new_repo" => "새 저장소".to_string(),
        "nav.create" => "생성".to_string(),
        // Auth
        "auth.sign_in" => "로그인".to_string(),
        "auth.sign_out" => "로그아웃".to_string(),
        "auth.register" => "회원가입".to_string(),
        "auth.username" => "사용자 이름".to_string(),
        "auth.email" => "이메일".to_string(),
        "auth.password" => "비밀번호".to_string(),
        "auth.display_name" => "표시 이름".to_string(),
        "auth.forgot_password" => "비밀번호를 잊으셨나요?".to_string(),
        // Settings
        "settings.title" => "설정".to_string(),
        "settings.language" => "언어".to_string(),
        // Repo
        "repo.clone_url" => "클론 URL".to_string(),
        "repo.description" => "설명".to_string(),
        "repo.visibility" => "공개 범위".to_string(),
        "repo.public" => "공개".to_string(),
        "repo.private" => "비공개".to_string(),
        // Common
        "common.save" => "저장".to_string(),
        "common.cancel" => "취소".to_string(),
        "common.delete" => "삭제".to_string(),
        "common.edit" => "편집".to_string(),
        "common.search" => "검색".to_string(),
        "common.loading" => "로딩 중...".to_string(),
        "common.error" => "오류".to_string(),
        "common.success" => "성공".to_string(),
        "common.back" => "뒤로".to_string(),
        "common.next" => "다음".to_string(),
        "common.previous" => "이전".to_string(),
        "common.confirm" => "확인".to_string(),
        // Footer
        "footer.version" => "버전".to_string(),
        _ => key.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ko_translations() {
        assert_eq!(get("nav.home"), "홈");
        assert_eq!(get("auth.sign_in"), "로그인");
        assert_eq!(get("common.save"), "저장");
    }
}
