// Placeholder — interactivity added in later phases
document.addEventListener('DOMContentLoaded', () => {
    // Flash message auto-dismiss
    document.querySelectorAll('.flash-message').forEach(el => {
        setTimeout(() => { el.style.opacity = '0'; el.style.transition = 'opacity 0.3s'; }, 5000);
    });

    // Captcha refresh button
    const captchaRefresh = document.getElementById('captcha-refresh');
    if (captchaRefresh) {
        captchaRefresh.addEventListener('click', async () => {
            const img = document.getElementById('captcha-img');
            const token = document.getElementById('captcha-token');
            try {
                const resp = await fetch('/auth/captcha/refresh', { method: 'POST' });
                const data = await resp.json();
                img.src = data.image;
                token.value = data.token;
            } catch (e) { console.error('captcha refresh failed', e); }
        });
    }
});
