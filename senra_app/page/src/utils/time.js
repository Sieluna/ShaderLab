export function parseTime(time) {
    const string = String(time).trim();

    // Unix timestamp in milliseconds
    if (/^\d{11,}$/.test(string)) return new Date(+string);
    // Unix timestamp in seconds
    if (/^\d{9,10}$/.test(string)) return new Date(+string * 1e3);

    // ISO / RFC‑3339 → native browser date
    if (/^\d{4}-\d{2}-\d{2}T\d{2}:/.test(string)) return new Date(string);

    // RFC‑2822 / HTTP‑date
    if (/^[A-Z][a-z]{2}, \d{2}/.test(string)) return new Date(string);

    // TIMESTAMPTZ 2025-05-08 17:29:50.0 +00:00:00
    if (/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d [+-]\d{2}:\d{2}:\d{2}$/.test(string)) {
        const iso = string
            .replace(' ', 'T')
            .replace(/\.0\b/, '')
            .replace(/ ([+-]\d{2}):(\d{2}):(\d{2})$/, '$1:$2');
        return new Date(iso);
    }

    // TIMESTAMP (no timezone)
    if (/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(\.\d+)?$/.test(string)) {
        return new Date(string.replace(' ', 'T'));
    }

    return new Date(string);
}

export function time(input, now = new Date(), locale = 'default') {
    const d = input instanceof Date ? input : parseTime(input);
    if (isNaN(d)) return null;

    const sameYear = d.getFullYear() === now.getFullYear();
    const sameDay = sameYear && d.getMonth() === now.getMonth() && d.getDate() === now.getDate();

    if (sameDay) {
        // Same day: only show time
        return d.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit' });
    }
    if (sameYear) {
        // Same year: show month/day
        return d.toLocaleDateString(locale, { month: '2-digit', day: '2-digit' });
    }

    // Different year: show year/month
    return d.toLocaleDateString(locale, { year: 'numeric', month: '2-digit' });
}
