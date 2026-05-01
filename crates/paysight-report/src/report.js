// PaySight HTML report client-side animations.

(function () {
    'use strict';

    // Reveal cards as they enter the viewport.
    const cards = document.querySelectorAll('.card');
    const observer = new IntersectionObserver(
        (entries) => {
            for (const entry of entries) {
                if (entry.isIntersecting) {
                    entry.target.classList.add('revealed');
                    animateScores(entry.target);
                    observer.unobserve(entry.target);
                }
            }
        },
        { threshold: 0.12, rootMargin: '0px 0px -80px 0px' }
    );
    cards.forEach((c) => observer.observe(c));

    // Trigger the score-bar fill once a card is visible. Each `.score-fill`
    // exposes its target percentage via `data-pct`; we transition the
    // inline `width` into place on the next animation frame.
    function animateScores(card) {
        const fills = card.querySelectorAll('.score-fill');
        fills.forEach((el, i) => {
            const pct = el.getAttribute('data-pct') || '0';
            setTimeout(() => {
                el.style.width = pct + '%';
            }, 100 + i * 40);
        });
    }
})();
