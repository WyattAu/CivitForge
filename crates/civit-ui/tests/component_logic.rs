use civit_ui::components::{BadgeColor, InputType};

#[test]
fn badge_color_class_contains_expected_prefix() {
    let cls = BadgeColor::Success.class();
    assert!(
        cls.contains("bg-green-100"),
        "Success should have green bg: {cls}"
    );

    let cls = BadgeColor::Warning.class();
    assert!(
        cls.contains("bg-yellow-100"),
        "Warning should have yellow bg: {cls}"
    );

    let cls = BadgeColor::Danger.class();
    assert!(
        cls.contains("bg-red-100"),
        "Danger should have red bg: {cls}"
    );

    let cls = BadgeColor::Info.class();
    assert!(
        cls.contains("bg-blue-100"),
        "Info should have blue bg: {cls}"
    );

    let cls = BadgeColor::Neutral.class();
    assert!(
        cls.contains("bg-gray-100"),
        "Neutral should have gray bg: {cls}"
    );
}

#[test]
fn badge_color_class_contains_dark_mode() {
    for color in [
        BadgeColor::Success,
        BadgeColor::Warning,
        BadgeColor::Danger,
        BadgeColor::Info,
        BadgeColor::Neutral,
    ] {
        let cls = color.class();
        assert!(
            cls.contains("dark:"),
            "BadgeColor::{color:?} should include dark mode: {cls}"
        );
    }
}

#[test]
fn badge_color_all_classes_unique() {
    let classes = [
        BadgeColor::Success.class(),
        BadgeColor::Warning.class(),
        BadgeColor::Danger.class(),
        BadgeColor::Info.class(),
        BadgeColor::Neutral.class(),
    ];
    let mut sorted = classes.to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 5, "All BadgeColor classes should be unique");
}

#[test]
fn input_type_default_is_text() {
    assert_eq!(InputType::default(), InputType::Text);
}

#[test]
fn input_type_partial_eq() {
    assert_eq!(InputType::Text, InputType::Text);
    assert_eq!(InputType::Email, InputType::Email);
    assert_ne!(InputType::Text, InputType::Password);
    assert_ne!(InputType::Textarea, InputType::Select);
}

#[test]
fn pagination_params_offset_priority_over_page() {
    let p = civit_shared::PaginationParams {
        per_page: Some(10),
        page: Some(3),
        offset: Some(50),
    };
    assert_eq!(
        p.effective_offset(),
        50,
        "Explicit offset should take priority"
    );
}

#[test]
fn pagination_page_saturating_sub() {
    let p = civit_shared::PaginationParams {
        per_page: Some(10),
        page: Some(0),
        offset: None,
    };
    assert_eq!(
        p.effective_offset(),
        0,
        "Page 0 should saturate to offset 0"
    );
}

#[test]
fn pagination_total_pages_div_ceil() {
    let params = civit_shared::PaginationParams {
        per_page: Some(10),
        page: Some(1),
        offset: None,
    };
    let pag = civit_shared::Pagination::from_total(21, &params);
    assert_eq!(pag.total_pages, 3);

    let pag = civit_shared::Pagination::from_total(20, &params);
    assert_eq!(pag.total_pages, 2);

    let pag = civit_shared::Pagination::from_total(1, &params);
    assert_eq!(pag.total_pages, 1);
}

#[test]
fn pagination_boundary_pages() {
    let pag = civit_shared::Pagination {
        page: 1,
        per_page: 10,
        total: 100,
        total_pages: 10,
    };
    assert!(!pag.has_prev());
    assert!(pag.has_next());

    let pag = civit_shared::Pagination {
        page: 10,
        per_page: 10,
        total: 100,
        total_pages: 10,
    };
    assert!(pag.has_prev());
    assert!(!pag.has_next());
}

#[test]
fn user_role_all_ranks_unique() {
    let ranks = [
        (civit_shared::UserRole::Owner, 60u8),
        (civit_shared::UserRole::Admin, 50),
        (civit_shared::UserRole::Maintainer, 40),
        (civit_shared::UserRole::Developer, 30),
        (civit_shared::UserRole::Reporter, 20),
        (civit_shared::UserRole::Guest, 10),
    ];
    let mut values: Vec<u8> = ranks.iter().map(|r| r.0.rank()).collect();
    values.sort();
    values.dedup();
    assert_eq!(values.len(), 6, "All UserRole ranks should be unique");
    for (role, expected) in ranks {
        assert_eq!(role.rank(), expected, "{role:?} rank mismatch");
    }
}
