document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
        document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
        btn.classList.add('active');
        document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
    });
});

const featureDesc = document.getElementById('feature-desc');
document.querySelectorAll('.feature-tags button').forEach(btn => {
    btn.addEventListener('click', () => {
        const wasActive = btn.classList.contains('active');
        document.querySelectorAll('.feature-tags button.active').forEach(b => b.classList.remove('active'));
        if (wasActive) {
            featureDesc.innerHTML = '';
        } else {
            btn.classList.add('active');
            featureDesc.innerHTML = btn.dataset.desc;
        }
    });
});
