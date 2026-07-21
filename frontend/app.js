'use strict';

const API_URL = 'http://localhost:8080/api/quote';

const button = document.getElementById('get-quote-btn');
const quoteText = document.getElementById('quote-text');
const quoteAuthor = document.getElementById('quote-author');
const status = document.getElementById('status');

async function fetchQuote() {
  setLoading(true);
  status.textContent = '';

  try {
    const response = await fetch(API_URL, { method: 'GET' });

    if (!response.ok) {
      const body = await response.json().catch(() => ({}));
      throw new Error(
        body.error || `Request failed with status ${response.status}`
      );
    }

    const data = await response.json();
    quoteText.textContent = `"${data.quote}"`;
    quoteAuthor.textContent = `— ${data.author}`;
  } catch (err) {
    status.textContent = `Something went wrong: ${err.message}`;
  } finally {
    setLoading(false);
  }
}

function setLoading(isLoading) {
  button.disabled = isLoading;
  button.textContent = isLoading
    ? 'Consulting five microservices…'
    : 'Get Quote';
}

button.addEventListener('click', fetchQuote);
