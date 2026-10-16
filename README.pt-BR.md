# keepclip

[![CI](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml/badge.svg)](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml)

[Read in English](README.md)

Salve um vídeo a partir de um link copiado, sem sites de download cheios de
anúncios: copie o link, abra o keepclip, escolha a qualidade pelo tamanho e o
arquivo vai para a sua pasta de downloads. Funciona com os
[sites suportados pelo yt-dlp](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md)
e fecha quando termina: nada fica rodando em segundo plano.

> **Baixe só o que você tem direito de guardar:** seus próprios vídeos,
> vídeos com licença que permita ou vídeos cujo dono deixa salvar. Respeite os
> direitos autorais e os termos de cada site. O keepclip é uma ferramenta; o
> que você faz com ela é com você.

O keepclip está na primeira versão, que roda numa janela de console. O que
muda em cada versão está no [changelog](CHANGELOG.md) (em inglês).

## Como funciona

1. Copie o link de um vídeo.
2. Abra o keepclip com `--clipboard`, por um atalho, um lançador ou uma ação
   do mouse (veja [Abrir por outro app](#abrir-por-outro-app)).
3. O keepclip pergunta ao yt-dlp o que o link oferece e mostra as opções, com
   o tamanho de cada uma:

   ```
   Sample video | Sample channel | 10:35

      1) 1080p 60 fps      255.5 MB
      2) 720p 60 fps       153.3 MB
      3) 480p              36.7 MB
      4) Audio only (MP3)  ~14.5 MB

   Choose (1-4, Enter = 1, 0 = cancel):
   ```

4. Escolha uma. O keepclip mostra o progresso, salva o arquivo e mostra uma
   notificação do Windows; clique nela para abrir a pasta.

## Instalação

Ainda não há downloads prontos: compile o keepclip a partir do código, como
explica [Compilar a partir do código](#compilar-a-partir-do-código). Ele
precisa do Windows 10 ou 11, 64 bits.

### Primeira execução

O keepclip pergunta onde salvar os downloads e guarda a resposta no arquivo
de configurações.

Depois ele baixa, uma vez só, os programas gratuitos que fazem o trabalho,
para `%LOCALAPPDATA%\keepclip\bin`. Ele pergunta antes de baixar, pega cada
um da release oficial no GitHub e confere com o checksum SHA-256 publicado
junto:

| Programa | O que faz | De onde vem | Licença |
|---|---|---|---|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | lê o link e baixa o vídeo | `yt-dlp/yt-dlp` | Unlicense |
| [ffmpeg](https://ffmpeg.org) | junta vídeo e som, gera MP3 | `yt-dlp/FFmpeg-Builds`, as builds feitas para o yt-dlp | GPL |
| [Deno](https://deno.com) | roda o JavaScript que alguns sites pedem | `denoland/deno` | MIT |

São uns 150 MB de download, e os programas ocupam uns 310 MB no disco. O
keepclip não inclui nem distribui nenhum deles. Os sites mudam com
frequência, então o keepclip atualiza o yt-dlp (`yt-dlp -U`) quando a última
verificação tem mais de 7 dias.

### Desinstalar

Apague o `keepclip.exe` e depois as pastas `%APPDATA%\keepclip`
(configurações) e `%LOCALAPPDATA%\keepclip` (os programas baixados). Para
tirar o nome que o keepclip registra para as notificações, apague a chave do
registro `HKEY_CURRENT_USER\Software\Classes\AppUserModelId\keepclip`. Seus
vídeos ficam na pasta de downloads.

## Usando o keepclip

```bash
keepclip --clipboard
```

baixa o link da área de transferência. As outras formas de abrir:

| Comando | Faz |
|---|---|
| `keepclip <link>` | baixa esse link |
| `keepclip` | usa o link da área de transferência ou pede para colar um |
| `keepclip --choose-folder` | escolhe de novo a pasta de downloads |
| `keepclip --dir <pasta>` | guarda o arquivo de configurações nessa pasta, útil para testes |

**As opções.** Há uma linha por resolução, da maior para a menor, e depois
**Audio only (MP3)** (só o áudio). Em cada resolução o keepclip escolhe o
vídeo que toca em quase todo lugar, H.264 com som AAC num arquivo MP4, quando
o site tem; quando não tem, como em alguns vídeos acima de 1080p, escolhe o
melhor que houver e o arquivo pode sair em MKV. O MP3 é de 192 kbps, então o
tamanho é conhecido de antemão. "~" marca um tamanho que o site só estimou, e
"size unknown", um tamanho que ele não informou.

**Os nomes dos arquivos** seguem `<quem postou> - <título> [<id do vídeo>].<ext>`,
com os caracteres que o Windows não aceita trocados, e são cortados em 120
caracteres. Um vídeo que já está na pasta não é baixado de novo.

**Um link com vários vídeos**, como um post com mais de um, mostra antes a
lista deles (até 20) e baixa o que você escolher.

**Quando algo dá errado**, o keepclip mostra o motivo que o yt-dlp deu e
deixa a janela aberta até você apertar Enter.

### Abrir por outro app

O keepclip foi feito para ser aberto por outro programa e fechar quando
termina. Qualquer lançador, atalho de teclado ou ação do mouse que rode um
programa pode abri-lo com o link copiado:

```
"C:\caminho\para\keepclip.exe" --clipboard
```

Quando a janela do console é só dele, o keepclip a fecha dois segundos depois
do download; a notificação fica na central de notificações do Windows. Se o
Windows não mostrar a notificação, a janela espera um Enter.

## Configuração

O arquivo de configurações é `%APPDATA%\keepclip\config.toml`. O keepclip o
escreve quando você escolhe a pasta de downloads, e você pode editá-lo:

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
- **Só console**, em inglês. Uma janela vem numa versão futura.
- **Só Windows** por enquanto. A lógica em `core/` não tem código do Windows;
  em outros sistemas o keepclip espera `yt-dlp`, `ffmpeg` e `deno` no PATH,
  mas não foi testado lá.
- **Títulos longos**, como posts cheios de hashtags e emoji, viram nomes de
  arquivo longos, cortados em 120 caracteres.

## Privacidade

O keepclip não tem telemetria. Ele se conecta ao GitHub para baixar e
atualizar os programas e, pelo yt-dlp, aos sites dos links que você der. Nada
além disso.

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

Depois rode `target\release\keepclip.exe`.

## Organização do projeto

```
core/   biblioteca (keepclip-core): lógica pura e testável, sem código do
        Windows: lê o que o yt-dlp informa, monta o menu, os argumentos do
        download e o arquivo de configurações, e acompanha o progresso
cli/    o programa (keepclip.exe): console, área de transferência,
        notificação, e o download e a atualização de yt-dlp, ffmpeg e Deno
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

### Contribuição

A menos que você diga explicitamente o contrário, qualquer contribuição que
você enviar intencionalmente para inclusão no projeto, como definido na
licença Apache-2.0, terá licença dupla como acima, sem termos ou condições
adicionais.
