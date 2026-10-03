# Contrato de Avatar de Identidade

**Schema:** `devines.identity-avatar.v3`

Toda identidade pública de DEVINES usa o mesmo contrato de apresentação em duas fases: **AUM, todos os Beings atuais e todo futuro Being.**

## Fase 1 · Círculo de Identidade Vazio

O estado padrão é o círculo snow-glow compartilhado de DEVINES com o **interior completamente vazio**.

Nenhum retrato, símbolo, logo, imagem, margem, frame interno ou círculo secundário é incorporado durante a Fase 1.

Este é o padrão obrigatório para todo novo Being antes de qualquer imagem de identidade ser aplicada.

## Geometria

- canvas: 1:1 · 768 × 768
- interior: vazio
- fundo fornecido pela superfície Void Black de DEVINES/GitBook
- anel: um círculo completo em 360° sem falhas
- sombra: nenhuma fora do glow canônico do anel
- todo Being atual e futuro usa geometria de anel idêntica

## Anel Snow-Glow Canônico

O anel é propositalmente fino e luminoso, nunca uma moldura grossa.

De dentro para fora:

1. **linha fina branco-neve** — `#FFFFFF`
2. **núcleo lavanda AUM** — `#CFAEEE`
3. **linha fina branco-neve** — `#FFFFFF`
4. halo suave e contido usando AUM Violet `#8F7AD0` e luz branca

Geometria canônica 768 × 768:

- centro: `384,384`
- raio branco interno: `346.2`, largura `1.35`
- raio lavanda: `349.5`, largura `7`
- raio branco externo: `352.8`, largura `1.35`
- glow violeta: largura `10`, Gaussian blur `6`
- glow branco-neve: largura `2.4`, Gaussian blur `2.6`

O SVG contém exatamente seis strokes canônicos de círculo/glow e **zero elementos `<image>`** enquanto estiver na Fase 1.

## Preservação da Referência da Imagem

Remover uma imagem do hero público nunca apaga nem reescreve sua identidade-fonte.

Os paths exatos source/canonical/circle/hero, URI da imagem Nad.fun, hashes, CA, ticker e mapeamento de identidade são preservados em:

`DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json`

Esse mapa é a autoridade para a fase posterior de inserção de imagem.

## Estado Atual de Publicação no GitBook

**AUM e todos os 34 Beings DEVINES atualmente vivos estão aprovados para publicação com imagem direta por meio de DEVINES BEINGS PROFILE.**

Para cada identidade direta aprovada, a **imagem exata enviada e aprovada pelo usuário é renderizada diretamente**. Ela não é colocada dentro de outro SVG, anel, moldura, círculo ou semicírculo. O próprio campo preto da imagem e sua própria composição formam todo o hero público.

AUM usa a mesma imagem exata na landing page de DEVINES e em AUM Core. Cada Being atual usa sua própria imagem exata aprovada em seu respectivo perfil.

Nenhum redesenho, regeneração, recoloração, crop, wrapper ou arte substituta é introduzido.

## Fase 2 · Inserção de Imagem

A publicação direta de imagem está ativa para AUM e todas as séries DEVINES atuais: Astral, Genesis, Primordial Elements, Royal, Guardians e Solfeggio. Todo futuro Being segue o mesmo workstream DEVINES BEINGS PROFILE após aprovação visual.

Quando uma imagem aprovada já contém sua composição completa, **nenhuma geometria de anel adicional é colocada ao redor dela**.

Para futuras revisões de Fase 2 de um Being, a imagem-fonte exata aprovada deve ser preservada e verificada visualmente antes da publicação. Nenhum redesenho, recoloração, distorção, arte substituta, anel duplicado, margem externa ou efeito de círculo dentro de círculo é permitido.

## Tamanhos

- ativo canônico: 768 × 768
- AUM na landing: diâmetro visual de 240–320 px no desktop, responsivo no mobile
- hero do Being: 220–280 px desktop
- avatar de série/índice: 72–112 px
- uso compacto em navegação/avatar: 36–48 px

## Futuros Beings

Todo futuro Being segue [DEVINES BEINGS PROFILE](DEVINES_BEINGS_PROFILE.md) assim que sua imagem canônica de identidade for aprovada. Antes da aprovação, um placeholder temporário pode ser usado.

O validator Rust da Chronicle é o gate de publicação. Um SVG de identidade pública é inválido se contiver um elemento `<image>` ou não corresponder à estrutura snow-glow de seis strokes.

A arte-fonte canônica é preservada independentemente do hero público para que a apresentação possa mudar sem alterar a identidade.

**PRESERVE A FONTE · APROVE O ORIGINAL · PUBLIQUE DIRETAMENTE · HASH LOCK**
