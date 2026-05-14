/* ==========================================
   ArcBid — App Logic
   Wallet connection, timers, modals, canvas
   ========================================== */

'use strict';

// ── STATE ────────────────────────────────────────
let walletConnected = false;
let walletAddress = null;
let walletType = null;

// ── CANVAS BACKGROUND ────────────────────────────
(function initCanvas() {
  const canvas = document.getElementById('bg-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  let W, H, particles = [];

  function resize() {
    W = canvas.width = window.innerWidth;
    H = canvas.height = window.innerHeight;
  }
  window.addEventListener('resize', resize);
  resize();

  class Particle {
    constructor() { this.reset(); }
    reset() {
      this.x = Math.random() * W;
      this.y = Math.random() * H;
      this.r = Math.random() * 1.5 + 0.3;
      this.vx = (Math.random() - 0.5) * 0.25;
      this.vy = (Math.random() - 0.5) * 0.25;
      this.alpha = Math.random() * 0.6 + 0.1;
      this.color = Math.random() > 0.5 ? '124,92,255' : '160,124,255';
    }
    update() {
      this.x += this.vx; this.y += this.vy;
      if (this.x < 0 || this.x > W || this.y < 0 || this.y > H) this.reset();
    }
    draw() {
      ctx.beginPath();
      ctx.arc(this.x, this.y, this.r, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(${this.color},${this.alpha})`;
      ctx.fill();
    }
  }

  for (let i = 0; i < 120; i++) particles.push(new Particle());

  function draw() {
    ctx.clearRect(0, 0, W, H);
    // Connection lines
    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const dx = particles[i].x - particles[j].x;
        const dy = particles[i].y - particles[j].y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 100) {
          ctx.beginPath();
          ctx.moveTo(particles[i].x, particles[i].y);
          ctx.lineTo(particles[j].x, particles[j].y);
          ctx.strokeStyle = `rgba(124,92,255,${0.06 * (1 - dist / 100)})`;
          ctx.lineWidth = 0.5;
          ctx.stroke();
        }
      }
    }
    particles.forEach(p => { p.update(); p.draw(); });
    requestAnimationFrame(draw);
  }
  draw();
})();

// ── UTILITY ──────────────────────────────────────
function showToast(msg, type = 'success') {
  const toast = document.getElementById('toast');
  const toastMsg = document.getElementById('toastMsg');
  if (!toast || !toastMsg) return;
  toastMsg.textContent = msg;
  toast.classList.add('show');
  setTimeout(() => toast.classList.remove('show'), 3500);
}

function truncateAddress(addr) {
  return addr.slice(0, 4) + '...' + addr.slice(-4);
}

function generateFakeAddress() {
  const chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  return Array.from({ length: 44 }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
}

function fakeEncrypt(val) {
  if (!val || isNaN(val)) return 'Enter amount to preview encryption →';
  const hex = Array.from({ length: 48 }, () => Math.floor(Math.random() * 16).toString(16)).join('');
  return `0x${hex.slice(0, 8)}...${hex.slice(-8)} [ARC-MXE-SEALED]`;
}

// ── WALLET MODAL ─────────────────────────────────
function openWalletModal() {
  document.getElementById('walletModal').classList.add('active');
}
function closeWalletModal() {
  document.getElementById('walletModal').classList.remove('active');
}

document.getElementById('closeWalletModal')?.addEventListener('click', closeWalletModal);
document.getElementById('walletModal')?.addEventListener('click', function(e) {
  if (e.target === this) closeWalletModal();
});

function connectWallet(type) {
  closeWalletModal();
  walletConnected = true;
  walletType = type;
  walletAddress = generateFakeAddress();

  // Update all connect buttons
  const buttons = document.querySelectorAll('.btn-connect');
  buttons.forEach(btn => {
    btn.classList.add('connected');
    const icon = btn.querySelector('.btn-icon');
    const text = btn.querySelector('.btn-text');
    if (icon) icon.textContent = type === 'phantom' ? '👻' : '☀️';
    if (text) text.textContent = truncateAddress(walletAddress);
  });

  showToast(`${type === 'phantom' ? 'Phantom' : 'Solflare'} wallet connected!`);
}

document.getElementById('connectPhantom')?.addEventListener('click', () => connectWallet('phantom'));
document.getElementById('connectSolflare')?.addEventListener('click', () => connectWallet('solflare'));

// All connect wallet buttons
document.querySelectorAll('#connectWalletBtn, #mobileConnectBtn').forEach(btn => {
  btn.addEventListener('click', () => {
    if (walletConnected) {
      // Disconnect
      walletConnected = false;
      walletAddress = null;
      walletType = null;
      document.querySelectorAll('.btn-connect').forEach(b => {
        b.classList.remove('connected');
        const icon = b.querySelector('.btn-icon');
        const text = b.querySelector('.btn-text');
        if (icon) icon.textContent = '◈';
        if (text) text.textContent = 'Connect Wallet';
      });
      showToast('Wallet disconnected');
    } else {
      openWalletModal();
    }
  });
});

// ── BID MODAL ─────────────────────────────────────
let currentAuction = null;

function openBidModal(auctionName) {
  if (!walletConnected) {
    openWalletModal();
    return;
  }
  currentAuction = auctionName;
  const modal = document.getElementById('bidModal');
  const title = document.getElementById('bidModalTitle');
  const nameEl = document.getElementById('bidAuctionName');
  if (title) title.textContent = auctionName;
  if (nameEl) nameEl.textContent = auctionName.length > 24 ? auctionName.slice(0, 22) + '…' : auctionName;
  modal?.classList.add('active');
}

document.getElementById('closeBidModal')?.addEventListener('click', () => {
  document.getElementById('bidModal').classList.remove('active');
});
document.getElementById('bidModal')?.addEventListener('click', function(e) {
  if (e.target === this) this.classList.remove('active');
});

// Encryption preview
document.getElementById('bidAmount')?.addEventListener('input', function() {
  const preview = document.getElementById('encPreview');
  if (preview) preview.textContent = fakeEncrypt(this.value);
});

// Submit bid
document.getElementById('submitBidBtn')?.addEventListener('click', () => {
  const amount = document.getElementById('bidAmount')?.value;
  if (!amount || isNaN(amount) || parseFloat(amount) <= 0) {
    showToast('Please enter a valid bid amount');
    return;
  }
  // Simulate bid submission
  const btn = document.getElementById('submitBidBtn');
  btn.textContent = '🔐 Encrypting...';
  btn.style.opacity = '0.7';
  setTimeout(() => {
    btn.innerHTML = '<span class="lock-icon">🔐</span> Encrypt & Submit Bid';
    btn.style.opacity = '1';
    document.getElementById('bidAmount').value = '';
    document.getElementById('encPreview').textContent = 'Enter amount to preview encryption →';
    document.getElementById('bidModal').classList.remove('active');
    showToast(`Bid of ${parseFloat(amount).toFixed(2)} SOL encrypted & sealed!`);
  }, 1800);
});

// ── AUCTION TIMERS ─────────────────────────────────
function formatTime(seconds) {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return [h, m, s].map(n => String(n).padStart(2, '0')).join(':');
}

const timerEls = document.querySelectorAll('.timer-val[data-end]');
timerEls.forEach(el => {
  let seconds = parseInt(el.dataset.end);
  el.textContent = formatTime(seconds);
  const interval = setInterval(() => {
    seconds--;
    if (seconds <= 0) {
      clearInterval(interval);
      el.textContent = 'ENDED';
      el.style.color = 'var(--text3)';
    } else {
      el.textContent = formatTime(seconds);
      if (seconds < 300) el.style.color = 'var(--red)';
    }
  }, 1000);
});

// ── HAMBURGER MENU ─────────────────────────────────
const hamburger = document.getElementById('hamburger');
const mobileNav = document.getElementById('mobileNav');
hamburger?.addEventListener('click', () => {
  mobileNav?.classList.toggle('open');
});

// Close mobile nav on link click
mobileNav?.querySelectorAll('a').forEach(a => {
  a.addEventListener('click', () => mobileNav.classList.remove('open'));
});

// ── SCROLL ANIMATIONS ──────────────────────────────
const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.style.animationPlayState = 'running';
    }
  });
}, { threshold: 0.1 });

document.querySelectorAll('.step-card, .criteria-card, .auction-card').forEach(el => {
  el.style.animationPlayState = 'paused';
  observer.observe(el);
});

// ── ENCRYPTED BID DISPLAY SHIMMER ─────────────────
function shimmerEncValues() {
  document.querySelectorAll('.enc-value').forEach(el => {
    if (el.style.filter === 'blur(3px)') {
      const blocks = ['██', '███', '█████', '████', '██', '███████'];
      el.textContent = Array.from({ length: 3 }, () => blocks[Math.floor(Math.random() * blocks.length)]).join(' ');
    }
  });
}
setInterval(shimmerEncValues, 2500);
