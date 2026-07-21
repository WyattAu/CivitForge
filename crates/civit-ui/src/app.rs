#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages::*;
use crate::state::auth::provide_auth_context;
use crate::theme::Theme;

#[component]
pub fn App() -> impl IntoView {
    // ── Layer 0: Synchronous init (before Leptos mounts) ──
    // Theme: already applied by inline JS in index.html
    // Locale: already set by inline JS in index.html

    // ── Layer 1: Reactive providers ──
    provide_auth_context();
    let _i18n = crate::i18n::provide_i18n();

    // Initialize theme reactive state from DOM (already applied by inline JS)
    let initial_theme = Theme::from_storage_value(get_theme_from_storage().as_deref());
    let (theme_sig, set_theme) = signal(initial_theme);

    // Provide theme context for child components
    provide_context(ThemeContext {
        theme: theme_sig,
        set_theme,
    });

    view! {
        <Router>
            <div class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
                <a
                    href="#main-content"
                    class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-[100] focus:px-4 focus:py-2 focus:bg-blue-600 focus:text-white focus:rounded-md"
                >
                    "Skip to main content"
                </a>
                <crate::components::sidebar::Sidebar />
                <main id="main-content" class="lg:pl-64 min-h-screen flex flex-col">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 flex-1 w-full">
                        <Routes fallback=|| view! { <NotFoundPage /> }>
                            <Route path=path!("") view=HomePage />
                            <Route path=path!("/login") view=LoginPage />
                            <Route path=path!("/register") view=RegisterPage />
                            <Route path=path!("/repos") view=ReposPage />
                            <Route path=path!("/new-repo") view=NewRepoPage />
                            <Route path=path!("/activity") view=ActivityPage />
                            <Route path=path!("/orgs") view=OrgsPage />
                            <Route path=path!("/orgs/:id") view=OrgDetailPage />
                            <Route path=path!("/admin") view=AdminPage />
                            <Route path=path!("/admin/feature-flags") view=AdminFeatureFlagsPage />
                            <Route path=path!("/admin/dashboard") view=AdminDashboardPage />
                            <Route path=path!("/admin/site-settings") view=SiteSettingsPage />
                            <ParentRoute path=path!("/repos/:owner/:name") view=RepoDetailPage>
                                <Route path=path!("") view=CodePage />
                                <Route path=path!("code") view=CodePage />
                                <Route path=path!("code/*path") view=CodePage />
                                <Route path=path!("blame") view=BlamePage />
                                <Route path=path!("blame/*path") view=BlamePage />
                                <Route path=path!("commits") view=FileCommitsPage />
                                <Route path=path!("issues") view=IssuesPage />
                                <Route path=path!("issues/:number") view=IssueDetailPage />
                                <Route path=path!("wiki") view=WikiPage />
                                <Route path=path!("pipelines") view=PipelinesPage />
                                <Route path=path!("pulls") view=PullRequestsPage />
                                <Route path=path!("pulls/:number") view=PullRequestDetailPage />
                                <Route path=path!("pulls/:number/files") view=PullRequestDetailPage />
                                <Route path=path!("graph") view=GraphPage />
                                <Route path=path!("releases") view=ReleasesPage />
                                <Route path=path!("boards") view=BoardsPage />
                                <Route path=path!("discussions") view=DiscussionsPage />
                                <Route path=path!("pr-templates") view=PrTemplatesPage />
                                <Route path=path!("environments") view=EnvironmentsPage />
                                <Route path=path!("deployments") view=DeploymentsPage />
                                <Route path=path!("settings") view=RepoSettingsPage />
                                <Route path=path!("branch-protection") view=BranchProtectionPage />
                            </ParentRoute>
                            <Route path=path!("/settings") view=SettingsPage />
                            <Route path=path!("/profile") view=ProfilePage />
                            <Route path=path!("/profile/:username") view=ProfilePage />
                            <Route path=path!("/explore") view=ExplorePage />
                            <Route path=path!("/search") view=SearchPage />
                            <Route path=path!("*") view=NotFoundPage />
                        </Routes>
                    </div>
                    <crate::components::footer::Footer />
                </main>
            </div>
        </Router>
    }
}

/// Theme context provided to all child components.
#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: ReadSignal<Theme>,
    pub set_theme: WriteSignal<Theme>,
}

impl ThemeContext {
    /// Toggle theme: update signal, persist, apply DOM.
    pub fn toggle(&self) {
        let new = Theme::toggle_and_persist(self.theme.get_untracked());
        self.set_theme.set(new);
    }
}

fn get_theme_from_storage() -> Option<String> {
    #[cfg(feature = "csr")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("civit-theme").ok())
            .flatten()
    }
    #[cfg(not(feature = "csr"))]
    {
        None
    }
}
