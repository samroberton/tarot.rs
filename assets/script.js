function toggleNavigableSection(event) {
  const selectedSection = event.target.getAttribute('data-navigable');
  document.querySelectorAll(`section[data-navigable]`).forEach((section) => {
    section.hidden = section.getAttribute('data-navigable') !== selectedSection;
  });
}

function trimDefence() {
  const bidder = document.getElementById('bidder').value;
  const partner = document.getElementById('partner').value;
  const defenceSelect = document.getElementById('defence');
  defenceSelect.querySelectorAll('option').forEach((option) => {
    const shouldHide = [bidder, partner].includes(option.value);
    option.disabled = shouldHide;
    option.hidden = shouldHide;
  })
}

function countSelectedOptions(selectedOptions) {
  console.log(selectedOptions);
  var result = 0;
  for (const option of selectedOptions) {
    if (!!option.value && !option.disabled) {
      result++;
    }
  }
  return result;
}

function validateDefence() {
  const partner = document.getElementById('partner').value;
  console.log(`partner: '${partner}', !!partner ${!!partner}`);
  const defenceSelect = document.getElementById('defence');
  const playerCount = countSelectedOptions(defenceSelect.selectedOptions) + (!!partner ? 2 : 1);
  if (playerCount < 4 || playerCount > 5) {
    defenceSelect.setCustomValidity('S\'il vous plait selectionner la défense pour avoir 4 ou 5 joueurs en total');
    console.log(`Invalid - ${playerCount} players, with defence: ${defenceSelect.selectedOptions}`);
  } else {
    defenceSelect.setCustomValidity('');
    console.log(`Valid - ${playerCount} players, with defence: ${defenceSelect.selectedOptions}`);
  }
}

function updateWonText() {
  const bidder = document.getElementById('bidder').value;
  if (bidder) {
    document.getElementById('won-true').innerText = `Oui, ${bidder} a gagné(e) le contrat`;
    document.getElementById('won-false').innerText = `Non, ${bidder} a chuté(e) le contrat`;
  }
}

document.getElementById('bidder')?.addEventListener('input', () => updateWonText());

document.getElementById('bidder')?.addEventListener('input', () => trimDefence());
document.getElementById('partner')?.addEventListener('input', () => trimDefence());

document.getElementById('bidder')?.addEventListener('input', () => validateDefence());
document.getElementById('partner')?.addEventListener('input', () => validateDefence());
document.getElementById('defence')?.addEventListener('input', () => validateDefence());

document.getElementById('hand-form')?.addEventListener('submit', (event) => {
  const defenceSelect = document.getElementById('defence');
  if (!defenceSelect.validity.valid) {
    event.preventDefault();
  }
});
