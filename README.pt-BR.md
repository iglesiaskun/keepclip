# keepclip

[![CI](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml/badge.svg)](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml)

[Read in English](README.md)

Salve um vídeo a partir de um link copiado, sem sites de download cheios de
anúncios: copie o link, abra o keepclip pelo Menu Iniciar, escolha a
qualidade pelo tamanho e o arquivo vai para a sua pasta de downloads.
Funciona com os
[sites suportados pelo yt-dlp](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md)
e fecha quando termina: nada fica rodando em segundo plano.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/media/window-dark.png">
  <img alt="O keepclip aberto pelo Menu Iniciar com um link copiado no campo, e as opções de um vídeo de exemplo: quatro resoluções com o tamanho de cada uma, e só o áudio (MP3)" src="docs/media/window-light.png">
</picture>

> **Baixe só o que você tem direito de guardar:** seus próprios vídeos,
> vídeos com licença que permita ou vídeos cujo dono deixa salvar. Respeite os
> direitos autorais e os termos de cada site. O keepclip é uma ferramenta; o
> que você faz com ela é com você.

O keepclip está na primeira versão. Ele vem como uma janela pequena,
`keepclip.exe`, e como uma versão de console, `keepclip-cli.exe`, com as
mesmas opções. O que muda em cada versão está no [changelog](CHANGELOG.md)
(em inglês).

## Como funciona

1. Copie o link de um vídeo.
2. Abra o keepclip pelo Menu Iniciar. O link que você copiou já está no
   campo; ou cole um.
3. Clique em **Read link** (ler o link). O keepclip pergunta ao yt-dlp o que
   o link oferece e mostra o vídeo, com a miniatura, e as opções, com o
   tamanho de cada uma.
4. Clique numa, ou aperte o número dela. O keepclip mostra o progresso, salva
   o arquivo e oferece **Open folder** (abrir a pasta), **Play** (tocar) e
   **Download another** (baixar outro). A janela fecha sozinha cinco segundos
   depois, a menos que o mouse esteja sobre ela.

O keepclip também pode abrir por um atalho de teclado, um lançador ou um
botão do mouse e ler o link copiado na hora: veja
[Abrir por outro app](#abrir-por-outro-app).

## Instalação

Requisitos:

- Windows 10 ou 11, 64 bits.
- Para a janela, o WebView2 Runtime, a parte do Windows que mostra páginas
  web dentro dos apps. O Windows 11 já vem com ele, e o Windows Update já o
  instalou na maioria dos computadores com Windows 10. Onde ele falta, o
  `keepclip.exe` avisa, com o link para baixá-lo de graça:
  <https://developer.microsoft.com/microsoft-edge/webview2/>. A versão de
  console, `keepclip-cli.exe`, funciona sem ele.

Para instalar:

1. Baixe a versão mais recente na
   [página de Releases](https://github.com/iglesiaskun/keepclip/releases/latest):
   - **`keepclip-<versão>-windows-x64.msi`** (recomendado) instala o keepclip
     só para o seu usuário, sem pedir administrador, em
     `%LOCALAPPDATA%\Programs\keepclip`. Ele põe o keepclip no Menu Iniciar
     e a pasta dele no seu PATH, para o `keepclip-cli` funcionar em qualquer
     terminal aberto depois. Para atualizar, rode o `.msi` mais novo; um
     keepclip aberto fecha antes.
   - **`keepclip-<versão>-windows-x64.zip`** roda sem instalar: descompacte
     onde quiser e abra o `keepclip.exe`.
2. Se o Windows mostrar "O Windows protegeu o computador", clique em **Mais
   informações** e depois em **Executar assim mesmo**. Os arquivos ainda não
   têm assinatura de código, então o SmartScreen avisa sobre qualquer
   download novo do keepclip; os arquivos são gerados no GitHub a partir
   deste repositório, e você pode conferi-los, como explica
   [Conferir um download](#conferir-um-download).
3. Abra o **keepclip** pelo Menu Iniciar.

### Primeira execução

O keepclip pergunta onde salvar os downloads e guarda a resposta no arquivo
de configurações.

Depois ele baixa, uma vez só, os programas gratuitos que fazem o trabalho,
para `%LOCALAPPDATA%\keepclip\bin`. Ele pergunta antes de baixar (a janela
pergunta a pasta e os programas numa tela só), pega cada um da release
oficial no GitHub e confere com o checksum SHA-256 publicado junto:

| Programa | O que faz | De onde vem | Licença |
|---|---|---|---|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | lê o link e baixa o vídeo | `yt-dlp/yt-dlp` | Unlicense |
| [ffmpeg](https://ffmpeg.org) | junta vídeo e som, gera MP3 | `yt-dlp/FFmpeg-Builds`, as builds feitas para o yt-dlp | GPL |
| [Deno](https://deno.com) | roda o JavaScript que alguns sites pedem | `denoland/deno` | MIT |

São uns 150 MB de download, e os programas ocupam uns 310 MB no disco. O
keepclip não inclui nem distribui nenhum deles. Os sites mudam com
frequência, então o keepclip atualiza o yt-dlp (`yt-dlp -U`) quando a última
verificação tem mais de 7 dias.

### Conferir um download

Cada release lista o checksum SHA-256 dos arquivos em `SHA256SUMS`. Compare
a linha do seu arquivo com o que o PowerShell mostra para ele:

```powershell
Get-FileHash keepclip-0.1.0-windows-x64.msi
```

Os arquivos têm também um atestado de build, que diz que foram gerados pelo
workflow de release deste repositório. Com o [GitHub CLI](https://cli.github.com):

```powershell
gh attestation verify keepclip-0.1.0-windows-x64.msi --repo iglesiaskun/keepclip
```

### Desinstalar

**Instalado pelo `.msi`**: remova o keepclip em Configurações > Aplicativos >
Aplicativos instalados. Isso apaga também os programas que ele baixou e os
arquivos da própria janela (`%LOCALAPPDATA%\keepclip`), os pedaços de
downloads que não terminaram (`%TEMP%\keepclip`) e a pasta dele no seu
PATH. As configurações ficam em `%APPDATA%\keepclip`, para uma instalação
futura; apague essa pasta para tirar tudo.

**Pelo `.zip`**: apague a pasta dele e depois as pastas `%APPDATA%\keepclip`
(configurações) e `%LOCALAPPDATA%\keepclip` (os programas baixados e os
arquivos da própria janela), e `%TEMP%\keepclip` se existir (pedaços de
downloads que não terminaram). Para tirar o nome que o keepclip registra
para as notificações, apague a chave do registro
`HKEY_CURRENT_USER\Software\Classes\AppUserModelId\keepclip`.

Nos dois casos, seus vídeos ficam na pasta de downloads.

## Usando o keepclip

Aberto pelo Menu Iniciar, o keepclip mostra um campo para o link. Um link que
você copiou já está nele, e um link copiado com o keepclip aberto entra no
campo quando você volta para a janela, a menos que você tenha digitado algo
nele. Nada é lido até você clicar em **Read link** ou apertar Enter. A
engrenagem ao lado da pasta, na barra de baixo, abre as configurações: a
pasta de downloads e a versão de cada programa.

Pela linha de comando ou por um atalho:

| Comando | Faz |
|---|---|
| `keepclip` | abre com o link copiado no campo, pronto para ler |
| `keepclip --clipboard` | lê na hora o link da área de transferência |
| `keepclip <link>` | lê esse link na hora |
| `keepclip --choose-folder` | escolhe de novo a pasta de downloads; sozinho, abre as configurações |
| `keepclip --dir <pasta>` | guarda o arquivo de configurações nessa pasta, útil para testes |

O `keepclip-cli` aceita as mesmas opções; sem link, ele usa o da área de
transferência ou pede um. Na janela, as teclas 1 a 9 escolhem uma linha e
Esc fecha.

**As opções.** Há uma linha por resolução, da maior para a menor, e depois
**Audio only (MP3)** (só o áudio). Em cada resolução o keepclip escolhe o
vídeo que toca em quase todo lugar, H.264 com som AAC num arquivo MP4, quando
o site tem. Acima da maior resolução em H.264, como em alguns vídeos acima de
1080p, escolhe o melhor vídeo que houver, que pode tocar em menos lugares;
abaixo dela, resoluções sem H.264 ficam de fora. Os arquivos saem em MP4, ou
em MKV quando o vídeo e o som não cabem num MP4.

**O MP3** é de 192 kbps, feito com o som do próprio vídeo, então o tamanho é
conhecido de antemão pela duração.

**Tamanhos** com "~" são estimativas, do site ou pela taxa de bits e a
duração; "size unknown" quer dizer que o site não deu nenhum dos dois.

**Os nomes dos arquivos** seguem `<quem postou> - <título> [<id do vídeo>].<ext>`,
com o canal quando o site deixa vazio quem postou, os caracteres que o
Windows não aceita trocados e um corte em 120 caracteres. Os títulos vêm como
o site dá, com emoji e hashtags.

**Arquivos que já estão na pasta.** Quando a pasta já tem um arquivo com o
mesmo nome, como o mesmo vídeo em outra resolução ou o MP3 dele de antes, o
keepclip pergunta antes de baixar se quer substituir. A resposta é não, a
menos que você escolha **Replace** (no console, digite `y`). Se disser que
sim, o arquivo antigo fica como está até o novo terminar, então um download
que falhe não faz perder nada.

O keepclip monta cada arquivo em `%TEMP%\keepclip` e o move para a sua pasta
quando fica pronto. Pedaços de um download que parou no meio, com **Cancel**
ou por um erro, ficam lá, e o yt-dlp os aproveita quando você baixa a mesma
opção de novo.

**Um link com vários vídeos**, como um post com mais de um, mostra antes a
lista deles (até 20) e baixa o que você escolher.

**Quando algo dá errado**, o keepclip mostra o motivo que o yt-dlp deu,
pronto para copiar, com **Try again** (tentar de novo). O console deixa a
janela aberta até você apertar Enter.

**A notificação.** Quando um download termina com a janela fora da frente, o
keepclip mostra uma notificação do Windows; clique nela para abrir a pasta. O
console sempre mostra.

### Abrir por outro app

O keepclip também funciona como uma ação rápida: copie um link, aperte uma
tecla ou um botão do mouse, escolha a qualidade e pronto. Qualquer lançador,
atalho de teclado ou ferramenta do mouse que rode um programa pode abri-lo
com o link copiado:

```
"%LOCALAPPDATA%\Programs\keepclip\keepclip.exe" --clipboard
```

(Com o `.zip`, use a pasta onde você o descompactou.) A janela abre na hora,
no meio da tela onde está o mouse, já lendo o link e com o foco, então as
teclas dos números funcionam sem clicar. Quando a área de transferência não
tem link, ela mostra o campo para colar e lê um link copiado enquanto espera.

O `keepclip-cli.exe --clipboard` faz o mesmo num console. Quando a janela do
console é só dele, ele a fecha dois segundos depois do download; a
notificação fica na central de notificações do Windows. Se o Windows não
mostrar a notificação, o console espera um Enter.

## Configuração

O arquivo de configurações é `%APPDATA%\keepclip\config.toml`. O keepclip o
escreve quando você escolhe a pasta de downloads (**Change**, na barra de
baixo da janela ou nas configurações, escolhe outra), e você pode editá-lo:

```toml
version = 1

# Folder where downloads are saved.
download_dir = 'C:\Users\voce\Downloads'
```

`version = 1` é o formato do arquivo. Se uma versão futura mudar o formato,
o keepclip usa esse número para atualizar arquivos antigos. Um arquivo sem
ele conta como versão 1.

## Limitações conhecidas

- **Sites que pedem login** ainda não funcionam: o keepclip não entra na sua
  conta. Vídeos públicos funcionam.
- **Só em inglês.**
- **Só Windows** por enquanto. A lógica em `core/` não tem código do Windows;
  em outros sistemas o keepclip espera `yt-dlp`, `ffmpeg` e `deno` no PATH,
  mas não foi testado lá.
- **Títulos longos**, como posts cheios de hashtags e emoji, viram nomes de
  arquivo longos, cortados em 120 caracteres.
- **Ainda sem assinatura de código.** O SmartScreen do Windows e alguns
  antivírus avisam sobre programas novos sem assinatura e sem reputação. A
  assinatura está nos planos; até lá, [Conferir um download](#conferir-um-download)
  mostra como confirmar que um arquivo veio deste repositório.

## Privacidade

O keepclip não tem telemetria. Ele se conecta ao GitHub para baixar e
atualizar os programas, pelo yt-dlp aos sites dos links que você der e, para
a miniatura na janela, ao endereço da imagem que o yt-dlp informa. Nada além
disso.

## Compilar a partir do código

Requisitos (Windows):

- Rust estável com o target `x86_64-pc-windows-msvc` (instalado pelo `rustup`).
- Visual Studio ou Build Tools com a carga de trabalho **Desenvolvimento para
  desktop com C++** (traz o linker MSVC e o SDK do Windows).

```bash
cargo test
```

```bash
cargo build --release
```

Depois rode `target\release\keepclip.exe` (a janela) ou
`target\release\keepclip-cli.exe` (o console). A interface da janela, em
`window/ui`, é HTML, CSS e JavaScript puros: não há Node, npm nem outra
etapa de build.

Os arquivos das releases saem do `packaging/package.ps1`, que o workflow de
release roda; o comentário no topo dele explica como rodá-lo localmente. Ele
precisa do [WiX Toolset](https://wixtoolset.org) 5 para o `.msi` e do
[cargo-about](https://github.com/EmbarkStudios/cargo-about) para a lista de
licenças de terceiros.

## Organização do projeto

```
core/     biblioteca (keepclip-core): lógica pura e testável, sem código do
          Windows: lê o que o yt-dlp informa, monta o menu, os argumentos
          do download e o arquivo de configurações, e acompanha o progresso
system/   biblioteca (keepclip-system): o que os dois programas dividem:
          roda o yt-dlp, baixa e atualiza yt-dlp, ffmpeg e Deno, o arquivo
          de configurações, a área de transferência e a notificação
window/   a janela (keepclip.exe), em Tauri 2, com a interface em ui/
cli/      a versão de console (keepclip-cli.exe)
packaging/ o instalador (WiX), o .zip portátil e a lista de licenças de
          terceiros, gerados pelo package.ps1
```

## Contribuindo

Relatos de bugs, ideias e pull requests são bem-vindos. O [CONTRIBUTING.md](CONTRIBUTING.md)
explica como as mudanças entram. Problemas de segurança devem ser relatados
em particular, como descrito no [SECURITY.md](SECURITY.md).

## Licença

Licenciado sob uma das duas, à sua escolha:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) ou
  <https://www.apache.org/licenses/LICENSE-2.0>)
- Licença MIT ([LICENSE-MIT](LICENSE-MIT) ou <https://opensource.org/licenses/MIT>)

Os programas que o keepclip baixa na primeira execução (yt-dlp, ffmpeg, Deno)
não fazem parte do keepclip e mantêm as próprias licenças.

Os ícones da janela, em [`window/ui/icons`](window/ui/icons), vêm do
[Lucide](https://lucide.dev) e estão sob a licença ISC
([LICENSE-LUCIDE](window/ui/icons/LICENSE-LUCIDE)).

### Contribuição

A menos que você diga explicitamente o contrário, qualquer contribuição que
você enviar intencionalmente para inclusão no projeto, como definido na
licença Apache-2.0, terá licença dupla como acima, sem termos ou condições
adicionais.
