#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages::*;
use crate::state::auth::provide_auth_context;

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();
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
                <main id="main-content" class="lg:pl-64 min-h-screen">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <Routes fallback=|| view! { <NotFoundPage /> }>
                            <Route path=path!("") view=HomePage />
                            <Route path=path!("/login") view=LoginPage />
                            <Route path=path!("/register") view=LoginPage />
                            <Route path=path!("/repos") view=ReposPage />
                            <Route path=path!("/new-repo") view=NewRepoPage />
                            <Route path=path!("/activity") view=ActivityPage />
                            <ParentRoute path=path!("/repos/:owner/:name") view=RepoDetailPage>
                                <Route path=path!("") view=CodePage />
                                <Route path=path!("code") view=CodePage />
                                <Route path=path!("code/*path") view=CodePage />
                                <Route path=path!("issues") view=IssuesPage />
                                <Route path=path!("issues/:number") view=IssueDetailPage />
                                <Route path=path!("wiki") view=WikiPage />
                                <Route path=path!("pipelines") view=PipelinesPage />
                                <Route path=path!("pulls") view=PullRequestsPage />
                                <Route path=path!("pulls/:number") view=PullRequestDetailPage />
                                <Route path=path!("settings") view=RepoSettingsPage />
                            </ParentRoute>
                            <Route path=path!("/search") view=SearchPage />
                            <Route path=path!("/orgs") view=OrgsPage />
                            <Route path=path!("/orgs/:id") view=OrgDetailPage />
                            <Route path=path!("/settings") view=SettingsPage />
                            <Route path=path!("/explore") view=ExplorePage />
                            <Route path=path!("*") view=NotFoundPage />
                        </Routes>
                    </div>
                </main>
            </div>
        </Router>
    }
}
