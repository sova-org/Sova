(function () {
  function accordionPlugin(hook, vm) {
    hook.afterEach(function (html, next) {
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, 'text/html');

      const headers = doc.querySelectorAll('h2, h3, h4, h5, h6');

      headers.forEach(header => {
        const text = header.textContent.trim();

        if (text.endsWith(' +')) {
          header.textContent = text.slice(0, -2);
          header.classList.add('accordion-header');
          header.style.cursor = 'pointer';
          header.style.userSelect = 'none';

          const arrow = doc.createElement('span');
          arrow.className = 'accordion-arrow';
          arrow.textContent = ' ▼';
          header.appendChild(arrow);

          const contentWrapper = doc.createElement('div');
          contentWrapper.className = 'accordion-content';

          let sibling = header.nextElementSibling;
          const headerLevel = parseInt(header.tagName.substring(1));

          while (sibling) {
            const nextSibling = sibling.nextElementSibling;

            if (sibling.tagName.match(/^H[1-6]$/)) {
              const siblingLevel = parseInt(sibling.tagName.substring(1));
              if (siblingLevel <= headerLevel) {
                break;
              }
            }

            contentWrapper.appendChild(sibling.cloneNode(true));
            sibling.remove();
            sibling = nextSibling;
          }

          header.parentNode.insertBefore(contentWrapper, header.nextSibling);
        }
      });

      next(doc.body.innerHTML);
    });

    hook.doneEach(function () {
      const accordionHeaders = document.querySelectorAll('.accordion-header');

      accordionHeaders.forEach(header => {
        const content = header.nextElementSibling;
        const arrow = header.querySelector('.accordion-arrow');

        if (content && content.classList.contains('accordion-content')) {
          content.style.display = 'none';
          if (arrow) arrow.textContent = ' ▶';
        }

        header.onclick = function () {
          const content = this.nextElementSibling;
          const arrow = this.querySelector('.accordion-arrow');

          if (content && content.classList.contains('accordion-content')) {
            if (content.style.display === 'none') {
              content.style.display = 'block';
              if (arrow) arrow.textContent = ' ▼';
            } else {
              content.style.display = 'none';
              if (arrow) arrow.textContent = ' ▶';
            }
          }
        };
      });
    });
  }

  window.$docsify = window.$docsify || {};
  window.$docsify.plugins = [].concat(accordionPlugin, window.$docsify.plugins || []);
})();
