// Patch console methods to output structured JSON logs
// This runs before the Bun server starts to ensure all console output is formatted

const originalConsole = {
  log: console.log,
  error: console.error,
  warn: console.warn,
  info: console.info,
  debug: console.debug,
};

function formatLog(level, args) {
  const message = args.map(arg => 
    typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
  ).join(' ');
  
  const logEntry = {
    timestamp: new Date().toISOString(),
    level: level,
    message: message,
    target: 'bun',
  };
  
  originalConsole.log(JSON.stringify(logEntry));
}

console.log = (...args) => formatLog('info', args);
console.info = (...args) => formatLog('info', args);
console.warn = (...args) => formatLog('warn', args);
console.error = (...args) => formatLog('error', args);
console.debug = (...args) => formatLog('debug', args);
