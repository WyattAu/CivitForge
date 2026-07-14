CREATE TABLE IF NOT EXISTS graphql_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    query TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}',
    channel TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_graphql_subscriptions_user_id ON graphql_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_graphql_subscriptions_channel ON graphql_subscriptions(channel);
CREATE INDEX IF NOT EXISTS idx_graphql_subscriptions_enabled ON graphql_subscriptions(enabled);
