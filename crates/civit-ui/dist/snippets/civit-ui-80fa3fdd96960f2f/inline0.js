
export function installGlobalErrorListeners() {
    window.__civitforgeErrors = [];
    
    window.onerror = function(msg, url, line, col, error) {
        window.__civitforgeErrors.push({
            source: 'unhandled',
            message: String(msg),
            url: url || '',
            stack: error ? (error.stack || '') : '',
            timestamp: new Date().toISOString()
        });
        return false;
    };
    
    window.addEventListener('unhandledrejection', function(event) {
        window.__civitforgeErrors.push({
            source: 'unhandled_promise',
            message: String(event.reason),
            url: window.location.href,
            stack: event.reason && event.reason.stack ? event.reason.stack : '',
            timestamp: new Date().toISOString()
        });
    });
    
    var origConsoleError = console.error;
    console.error = function() {
        var args = Array.prototype.slice.call(arguments);
        origConsoleError.apply(console, args);
        window.__civitforgeErrors.push({
            source: 'console',
            message: args.map(function(a) { return typeof a === 'object' ? JSON.stringify(a) : String(a); }).join(' '),
            url: window.location.href,
            stack: '',
            timestamp: new Date().toISOString()
        });
    };
    
    var origConsoleWarn = console.warn;
    console.warn = function() {
        var args = Array.prototype.slice.call(arguments);
        origConsoleWarn.apply(console, args);
        window.__civitforgeErrors.push({
            source: 'console_warn',
            message: args.map(function(a) { return typeof a === 'object' ? JSON.stringify(a) : String(a); }).join(' '),
            url: window.location.href,
            stack: '',
            timestamp: new Date().toISOString()
        });
    };
}
