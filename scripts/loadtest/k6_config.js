export const config = {
  // Base URL for the API
  baseUrl: __ENV.BASE_URL || 'http://localhost:8080',
  
  // Authentication tokens
  authToken: __ENV.AUTH_TOKEN || '',
  
  // Thresholds
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests should be under 500ms
    http_req_failed: ['rate<0.01'],   // Less than 1% of requests should fail
  },
  
  // Default tags
  tags: {
    environment: __ENV.K6_ENVIRONMENT || 'development',
    service: 'civitforge-api',
  },
  
  // Timeouts
  timeouts: {
    http: '10s',
    gracePeriod: '5s',
  }
};

export default config;