function toggleNavigableSection(event) {
  selectedSection = event.target.getAttribute('data-navigable');
  document.querySelectorAll(`section[data-navigable]`).forEach((section) => {
    section.hidden = section.getAttribute('data-navigable') !== selectedSection;
  });
}