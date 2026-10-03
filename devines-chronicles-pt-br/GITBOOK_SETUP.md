# Configuração de Sync do GitBook

Este repositório está estruturado como fonte Git para DEVINES CHRONICLES.

1. No GitBook, criar ou selecionar o space público de DEVINES.
2. Conectar **Git Sync → GitHub**.
3. Selecionar `Devineverse/DEVINES-CHRONICLES` e a branch `main`.
4. Usar a raiz do repositório como content root.
5. Usar `README.md` como landing page.
6. Importar a navegação de `SUMMARY.md`.
7. Manter Git Sync bidirecional somente se edições feitas no GitBook também tiverem a intenção de se tornar histórico do repositório.

## Aparência Canônica do Site Inteiro

Aplicar estes valores em **Customization → site-wide** no GitBook para que toda página DEVINES compartilhe um único campo visual:

- Theme: **Clean**
- Default mode: **Dark**
- Tint color (dark): **#000000**
- Primary color (dark): **#CFAEEE**
- Sidebar: fundo **Default**, não Filled
- Links/active state: somente AUM Lavender / DEVINES Violet
- Corners: mínimos
- Shadows/depth: sutis ou inexistentes
- Não introduzir fundos de página cinza nem accents azuis do GitBook

Esse tint global é necessário para que o preto dentro dos ativos circulares de identidade de AUM e dos Beings se funda visualmente com a página ao redor, em vez de aparecer como um segundo retângulo preto.

Git Sync governa o manuscrito. O GitBook armazena o tema hospedado separadamente como configurações de customização do site.
