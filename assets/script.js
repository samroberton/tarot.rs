function toggleNavigableSection(event) {
  const selectedSection = event.target.closest('button')?.getAttribute('data-navigable');
  if (!selectedSection) {
    console.error('No data-navigable attribute found on button', { target: event.target, button: event.target.closest('button')});
    return;
  }
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
  const defenceSelect = document.getElementById('defence');
  const defenceCount = countSelectedOptions(defenceSelect.selectedOptions);

  if (partner) {
    if (defenceCount == 3) {
      defenceSelect.setCustomValidity('');
    } else {
      defenceSelect.setCustomValidity('Sélectionner 3 joueurs pour la défense');
    }
  } else {
    if (defenceCount == 3 || defenceCount == 4) {
      defenceSelect.setCustomValidity('');
    } else {
      defenceSelect.setCustomValidity('Sélectionner 3 ou 4 joueurs pour la défense pour avoir 4 ou 5 joueurs en total');
    }
  }
}

function updateWonText() {
  const bidder = document.getElementById('bidder').value;
  if (bidder) {
    document.getElementById('won-true').innerText = `Oui, ${bidder} a gagné le contrat`;
    document.getElementById('won-false').innerText = `Non, ${bidder} a perdu le contrat`;
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
