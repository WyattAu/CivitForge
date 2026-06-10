pub fn get(key: &str) -> String {
    match key {
        // App
        "app.name" => "CivitForge".to_string(),
        // Nav
        "nav.home" => "首页".to_string(),
        "nav.repos" => "仓库".to_string(),
        "nav.activity" => "动态".to_string(),
        "nav.explore" => "探索".to_string(),
        "nav.orgs" => "组织".to_string(),
        "nav.search" => "搜索".to_string(),
        "nav.new_repo" => "新建仓库".to_string(),
        "nav.create" => "创建".to_string(),
        // Auth
        "auth.sign_in" => "登录".to_string(),
        "auth.sign_out" => "退出登录".to_string(),
        "auth.register" => "注册".to_string(),
        "auth.username" => "用户名".to_string(),
        "auth.email" => "邮箱".to_string(),
        "auth.password" => "密码".to_string(),
        "auth.display_name" => "显示名称".to_string(),
        "auth.forgot_password" => "忘记密码？".to_string(),
        // Settings
        "settings.title" => "设置".to_string(),
        "settings.language" => "语言".to_string(),
        // Repo
        "repo.clone_url" => "克隆地址".to_string(),
        "repo.description" => "描述".to_string(),
        "repo.visibility" => "可见性".to_string(),
        "repo.public" => "公开".to_string(),
        "repo.private" => "私有".to_string(),
        // Common
        "common.save" => "保存".to_string(),
        "common.cancel" => "取消".to_string(),
        "common.delete" => "删除".to_string(),
        "common.edit" => "编辑".to_string(),
        "common.search" => "搜索".to_string(),
        "common.loading" => "加载中...".to_string(),
        "common.error" => "错误".to_string(),
        "common.success" => "成功".to_string(),
        "common.back" => "返回".to_string(),
        "common.next" => "下一步".to_string(),
        "common.previous" => "上一步".to_string(),
        "common.confirm" => "确认".to_string(),
        // Footer
        "footer.version" => "版本".to_string(),
        _ => key.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zh_translations() {
        assert_eq!(get("nav.home"), "首页");
        assert_eq!(get("auth.sign_in"), "登录");
        assert_eq!(get("common.save"), "保存");
    }
}
