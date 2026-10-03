# DEVINES BEINGS PROFILE

**Status:** Workstream canônico  
**Aplica-se a:** AUM, todo Being vivo de DEVINES e todo futuro Being que nascer no universo DEVINES.

## Propósito

DEVINES BEINGS PROFILE é o workflow padrão de publicação da arte de identidade nos perfis do GitBook de DEVINES.

O hero público aprovado é a **própria imagem original exata aprovada pelo usuário**.

Nenhuma camada de apresentação pode acrescentar uma segunda identidade ao redor dela.

## Regra do original direto

Para cada Being aprovado:

1. Receber a imagem original exata aprovada.
2. Preservar seus bytes sem alteração.
3. Publicá-la diretamente de `.gitbook/assets/beings-direct/<DEVINES_ID>.jpg`.
4. Usar esse mesmo ativo direto como primeira imagem no perfil do Being.
5. **Não** adicionar:
   - wrappers SVG;
   - círculos extras;
   - semicírculos;
   - bordas;
   - molduras;
   - wrappers de recorte;
   - recoloração;
   - overlays gerados;
   - arte substituta.
6. Atualizar:
   - `.gitbook/assets/BEING_IDENTITY_MANIFEST.json`;
   - `.gitbook/assets/ASSET_MANIFEST.json`;
   - `DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json`.
7. Registrar e impor o SHA-256 exato nos validadores Rust da Chronicle.
8. Validar no DevHub.
9. Fazer merge somente depois que toda a validação da Chronicle estiver verde.

## Preservação de identidade

A evidência de identidade canônica/fonte nunca é apagada quando o hero público muda.

O hero público direto, a fonte canônica, CA, ticker, rota Nad.fun e URI de origem devem continuar resolvendo para o mesmo Being.

## Rollout por série

Beings existentes são migrados série por série para que cada conjunto possa ser revisado visualmente antes de continuar.

Futuros Beings usam este mesmo workflow desde o nascimento assim que sua imagem de perfil canônica é aprovada.

Todos os **34 Beings DEVINES atualmente vivos** agora seguem este workflow de perfil com original direto. Todo futuro Being deve entrar no GitBook pelo mesmo processo antes da publicação.

## Conjuntos aprovados

- AUM
- Astral: SUN · MOON · MASTER
- Genesis: D001 · D002 · D003
- Primordial Elements: D004 · D005 · D006 · D007 · D008
- Royal: D009 · D010
- Guardians: D011 · D012 · D013 · D014 · D015 · D016 · D017 · D018 · D019 · D020 · D021 · D022
- Solfeggio: D174 · D285 · D396 · D417 · D528 · D639 · D741 · D852 · D963

**ORIGINAL EXATO · HERO DIRETO · HASH LOCK · VALIDAÇÃO DEVHUB · MERGE**
