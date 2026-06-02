#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
                <crate::components::sidebar::Sidebar />
                <main class="lg:pl-64 min-h-screen">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <Routes fallback=|| view! { <p>"Not found"</p> }>
                            <Route path=path!("") view=HomePage />
                            <Route path=path!("/login") view=LoginPage />
                            <Route path=path!("/register") view=LoginPage />
                            <Route path=path!("/repos") view=ReposPage />
                            <Route path=path!("/repos/:owner/:name") view=RepoDetailPage />
                            <Route path=path!("/repos/:owner/:name/issues") view=IssuesPage />
                            <Route path=path!("/repos/:owner/:name/wiki") view=WikiPage />
                            <Route path=path!("/repos/:owner/:name/settings") view=RepoSettingsPage />
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
