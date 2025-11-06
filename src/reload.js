(function() {
    const evtSource = new EventSource('/__reload__');
    evtSource.onmessage = function(e) {
        if (e.data === 'reload') {
            console.log('File changed, reloading...');
            location.reload();
        }
    };
    evtSource.onerror = function() {
        console.log('SSE connection error, retrying...');
        setTimeout(() => location.reload(), 1000);
    };
})();
