# keepclip

**Salve um vídeo a partir de um link copiado, no Windows, com o yt-dlp fazendo o trabalho.**

[![CI](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml/badge.svg)](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml)
[![Versão mais recente](https://img.shields.io/github/v/release/iglesiaskun/keepclip)](https://github.com/iglesiaskun/keepclip/releases/latest)
[![Licença: MIT ou Apache-2.0](https://img.shields.io/badge/license-MIT%20or%20Apache--2.0-blue)](#licença)

[Read in English](README.md)

![O keepclip abre pelo Menu Iniciar com um link copiado já no campo. Um clique em Read link mostra um vídeo de exemplo com a miniatura e cinco opções com o tamanho de cada uma; um clique em 720p mostra o vídeo e o som baixando e depois Saved, com Open folder, Play e Download another.](docs/media/download.gif)

Copie o link de um vídeo, abra o **keepclip** pelo Menu Iniciar e clique em
**Read link** (ler o link). Escolha a qualidade pelo tamanho, e o arquivo vai
para a sua pasta de downloads. Funciona com os
[sites suportados pelo yt-dlp](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md).

> **Baixe só o que você tem direito de guardar:** seus próprios vídeos,
> vídeos com licença que permita ou vídeos cujo dono deixa salvar. Respeite os
> direitos autorais e os termos de cada site. O keepclip é uma ferramenta; o
> que você faz com ela é com você.

## Por que usar

É pequeno: uma janela que abre quando você precisa e fecha assim que o
arquivo é salvo, sem nada rodando em segundo plano. Ele te poupa dos sites de
download cheios de anúncios e botões falsos. Não pede conta, não tem
telemetria nem anúncios, e é gratuito e de código aberto. Quem faz o trabalho
é o yt-dlp, o programa gratuito que acompanha os sites conforme eles mudam: o
keepclip instala, mantém atualizado e transforma o que ele encontra numa lista
curta de opções com o tamanho de cada uma.

## Recursos

### Uma lista curta de opções, com o tamanho de cada uma

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/media/window-dark.png">
  <img alt="O keepclip aberto pelo Menu Iniciar com um link copiado no campo, e as opções de um vídeo de exemplo: quatro resoluções com o tamanho de cada uma, e só o áudio (MP3)" src="docs/media/window-light.png">
</picture>

Há uma linha por resolução, da maior para a menor, e depois **Audio only
(MP3)** (só o áudio). Clique numa, ou aperte o número dela.

| Opção | O que você leva |
|---|---|
| Uma resolução, como 1080p ou 720p | o vídeo que toca em quase todo lugar: H.264 com som AAC, num arquivo MP4, quando o site tem |
| Acima da maior resolução em H.264, como em alguns vídeos acima de 1080p | o melhor vídeo que houver, que pode tocar em menos lugares |
| Audio only (MP3) | o som do próprio vídeo, num MP3 de 192 kbps |

Abaixo da maior resolução em H.264, as resoluções sem ele ficam de fora. Os
arquivos saem em MP4, ou em MKV quando o vídeo e o som não cabem num MP4.
Tamanhos com "~" são estimativas, do site ou pela taxa de bits e a duração;
"size unknown" quer dizer que o site não deu nenhum dos dois. O tamanho de um
MP3 é conhecido de antemão, pela duração.

### Abre com o seu link, fecha quando termina

Aberto pelo Menu Iniciar, o keepclip mostra um campo com o link que você
copiou já dentro. Nada é lido até você clicar em **Read link** ou apertar
Enter, e um link copiado com o keepclip aberto entra no campo quando você
volta para a janela, a menos que você tenha digitado algo nele.

Enquanto baixa, o keepclip mostra o progresso de cada parte, também no título
e no botão da barra de tarefas. **Cancel** (cancelar) para e guarda as partes:
escolher a mesma opção de novo continua de onde parou. Com o arquivo salvo, o
keepclip oferece **Open folder** (abrir a pasta), **Play** (tocar) e
**Download another** (baixar outro), e fecha sozinho cinco segundos depois, a
menos que o mouse esteja sobre a janela. Quando um download termina com a
janela fora da frente, uma notificação do Windows avisa; clique nela para
abrir a pasta.

A janela segue o modo claro ou escuro do Windows. As teclas 1 a 9 escolhem
uma linha, e Esc fecha.

### Seus arquivos ficam seguros

Quando a pasta já tem um arquivo com o mesmo nome, como o mesmo vídeo em
outra resolução ou o MP3 dele de antes, o keepclip pergunta antes de baixar se
quer substituir. A resposta é não, a menos que você escolha **Replace**
(substituir). Se disser que sim, o arquivo antigo fica como está até o novo
terminar, então um download que falhe não faz perder nada.

O keepclip monta cada arquivo em `%TEMP%\keepclip` e o move para a sua pasta
quando fica pronto, então a pasta nunca fica com meio vídeo. Os nomes dos
arquivos seguem `<quem postou> - <título> [<id do vídeo>].<ext>`, com o canal
quando o site deixa vazio quem postou, os caracteres que o Windows não aceita
trocados e um corte em 120 caracteres. Os títulos vêm como o site dá, com
emoji e hashtags.

### Links com vários vídeos, e erros que dizem o que fazer

Um link com vários vídeos, como um post com mais de um, mostra antes a lista
deles (até 20) e baixa o que você escolher.

Quando algo dá errado, o keepclip mostra o motivo que o yt-dlp deu, pronto
para copiar, com **Try again** (tentar de novo), e avisa quando o site parece
pedir login.

### Abrir por um atalho ou um botão do mouse

O keepclip também funciona como uma ação rápida: copie um link, aperte uma
tecla ou um botão do mouse, escolha a qualidade e pronto. Qualquer lançador,
atalho de teclado ou ferramenta do mouse que rode um programa pode abri-lo com
o link copiado, como uma ação no anel do
[Thumbring](https://github.com/iglesiaskun/thumbring):

```
"%LOCALAPPDATA%\Programs\keepclip\keepclip.exe" --clipboard
```

(Com o `.zip`, use a pasta onde você o descompactou.) A janela abre na hora,
no meio da tela onde está o mouse, já lendo o link e com o foco, então as
teclas dos números funcionam sem clicar. Quando a área de transferência não
tem link, ela mostra o campo para colar e lê um link copiado enquanto espera.

### Também em versão de console

O `keepclip-cli.exe` faz o mesmo num console, com as mesmas opções (a
["Referência do arquivo de configuração"](#referência-do-arquivo-de-configuração)
lista todas). Sem link, ele usa o da área de transferência ou pede um, e para
substituir um arquivo com o mesmo nome você digita `y`. Quando a janela do
console é só dele, ele a fecha dois segundos depois do download; a
notificação fica na central de notificações do Windows. Se o Windows não
mostrar a notificação, ou se algo der errado, o console espera um Enter.

## Instalação

O keepclip 0.1.0 é a primeira versão pública. O que mudou em cada versão está
no [changelog](CHANGELOG.md) (em inglês).

Requisitos:

- Windows 10 ou 11, 64 bits.
- Para a janela, o WebView2 Runtime, a parte do Windows que mostra páginas
  web dentro dos apps. Ele já vem com o Windows 11, e o Windows Update já o
  instalou na maioria dos computadores com Windows 10. Onde ele faltar, o
  `keepclip.exe` avisa, com o link para baixá-lo, de graça. A versão de
  console, `keepclip-cli.exe`, funciona sem ele.
- Internet: na primeira execução, o keepclip baixa os programas que fazem o
  trabalho, uns 150 MB (veja ["Primeiros passos"](#primeiros-passos)).

Para instalar:

1. Baixe a versão mais recente na
   [página de Releases](https://github.com/iglesiaskun/keepclip/releases/latest):
   - **`keepclip-<versão>-windows-x64.msi`** (recomendado) instala o keepclip
     só para o seu usuário, sem pedir permissão de administrador, em
     `%LOCALAPPDATA%\Programs\keepclip`. Ele põe o keepclip no Menu Iniciar e
     a pasta dele no seu PATH, para o `keepclip-cli` funcionar em qualquer
     terminal aberto depois. Para atualizar, rode o `.msi` mais novo; um
     keepclip aberto fecha antes.
   - **`keepclip-<versão>-windows-x64.zip`** roda sem instalar: descompacte
     onde quiser e abra o `keepclip.exe`.
2. O Windows provavelmente vai mostrar "O Windows protegeu o computador": os
   arquivos ainda não têm assinatura digital, e o SmartScreen avisa sobre
   programas novos que ainda não têm reputação. Clique em **Mais
   informações** e depois em **Executar assim mesmo**.
   ["Limitações conhecidas"](#limitações-conhecidas) explica por quê e como
   conferir se o download é o mesmo que o GitHub compilou a partir deste
   repositório.
3. Abra o **keepclip** pelo Menu Iniciar. Na primeira vez, ele pergunta onde
   salvar os seus vídeos e baixa os programas de que precisa, como
   ["Primeiros passos"](#primeiros-passos) explica.

**Desinstalar.** Com o `.msi`, remova o keepclip em Configurações >
Aplicativos > Aplicativos instalados. Isso apaga também os programas que ele
baixou e os arquivos da própria janela (`%LOCALAPPDATA%\keepclip`), os
pedaços de downloads que não terminaram (`%TEMP%\keepclip`) e a pasta dele no
seu PATH. As configurações ficam em `%APPDATA%\keepclip`, para uma instalação
futura; apague essa pasta para tirar tudo. Com o `.zip`, apague a pasta dele
e depois as pastas `%APPDATA%\keepclip` (configurações) e
`%LOCALAPPDATA%\keepclip` (os programas baixados e os arquivos da própria
janela), e `%TEMP%\keepclip` se existir. Para tirar o nome que o keepclip
registra para as notificações, apague a chave do registro
`HKEY_CURRENT_USER\Software\Classes\AppUserModelId\keepclip`. Nos dois casos,
seus vídeos ficam na pasta de downloads.

## Primeiros passos

1. **Copie o link** de um vídeo.
2. **Abra o keepclip** pelo Menu Iniciar. O link que você copiou já está no
   campo; ou cole um.
3. **Clique em Read link**, depois clique numa opção ou aperte o número dela.
   O arquivo vai para a sua pasta de downloads.

Na barra de baixo da janela, **Change** (trocar) escolhe outra pasta de
downloads. Ao lado, na tela com o campo do link, a engrenagem abre as
configurações: a pasta e a versão de cada programa.

**A primeira execução.** O keepclip pergunta onde salvar os downloads e, na
mesma tela, pelos programas gratuitos que fazem o trabalho. Ele os baixa uma
vez só, para `%LOCALAPPDATA%\keepclip\bin`, cada um da release oficial no
GitHub, e confere cada um com o checksum SHA-256 publicado junto:

| Programa | O que faz | De onde vem | Licença |
|---|---|---|---|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | lê o link e baixa o vídeo | `yt-dlp/yt-dlp` | Unlicense |
| [ffmpeg](https://ffmpeg.org) | junta vídeo e som, gera MP3 | `yt-dlp/FFmpeg-Builds`, as builds feitas para o yt-dlp | GPL |
| [Deno](https://deno.com) | roda o JavaScript que alguns sites pedem | `denoland/deno` | MIT |

São uns 150 MB de download, e os programas ocupam uns 310 MB no disco. O
keepclip não inclui nem distribui nenhum deles. Os sites mudam com
frequência, então o keepclip atualiza o yt-dlp (`yt-dlp -U`) quando a última
verificação tem mais de 7 dias.

## Limitações conhecidas

**Vídeos que pedem login.** O keepclip ainda não entra em conta nenhuma nos
sites, então vídeos privados, e vídeos que um site só mostra para quem está
logado, não baixam. Quando a resposta do yt-dlp parece isso, o keepclip avisa.
Vídeos públicos funcionam.

**Os sites mudam.** O keepclip lê cada site pelo yt-dlp, e os sites mudam com
frequência. Quando um muda, os downloads dele podem falhar até um yt-dlp novo
acompanhar, e o keepclip procura uma versão nova toda semana. Um download que
falha num site costuma ser assunto do yt-dlp: as
[issues do yt-dlp](https://github.com/yt-dlp/yt-dlp/issues) podem já ter o
caso.

**Uns 150 MB na primeira execução.** O keepclip não traz o yt-dlp, o ffmpeg
e o Deno: ele os baixa na primeira execução, uns 150 MB, que ocupam uns 310 MB
no disco. Sem internet, a primeira execução não termina. O yt-dlp se atualiza
toda semana; o ffmpeg e o Deno ficam como foram baixados da primeira vez.

**Avisos do SmartScreen e de antivírus.** Os arquivos das versões ainda não
têm assinatura digital: o certificado é caro para um projeto gratuito, e o
pedido de assinatura gratuita para projetos open source está em andamento,
como diz a [política de assinatura de código](#política-de-assinatura-de-código).
Até lá, o SmartScreen do Windows avisa sobre um programa novo e sem reputação ("O Windows protegeu o computador"), e um antivírus pode
desconfiar de um programa desconhecido que baixa outros programas e os roda,
que é exatamente o que o keepclip faz com o yt-dlp, o ffmpeg e o Deno. Para
conferir se o download é o mesmo que o GitHub compilou a partir deste
repositório:

- compare a saída de `Get-FileHash <arquivo>` no PowerShell com a linha desse
  arquivo no `SHA256SUMS` da mesma versão;
- ou, quando as notas da versão mencionarem um atestado de build, rode
  `gh attestation verify <arquivo> --repo iglesiaskun/keepclip` com o
  [GitHub CLI](https://cli.github.com).

Você também pode compilar o keepclip a partir do código, como o
[CONTRIBUTING.md](CONTRIBUTING.md#building-from-source) (em inglês) explica.

**Janela sem o WebView2.** Num computador sem o WebView2 Runtime (veja
"Instalação"), o `keepclip.exe` não abre; no lugar, ele diz onde baixar o
runtime. O `keepclip-cli.exe` funciona normalmente.

**Títulos longos.** Títulos cheios de hashtags e emoji viram nomes de arquivo
longos, cortados em 120 caracteres.

**Só Windows, em inglês.** Por enquanto o keepclip roda só no Windows, e a
janela e as mensagens dele estão em inglês. A lógica em `core/` não tem código
do Windows; em outros sistemas o keepclip espera `yt-dlp`, `ffmpeg` e `deno`
no PATH, mas não foi testado lá.

## Perguntas frequentes

**O keepclip coleta dados?**
Não. Ele não tem telemetria. Ele se conecta ao GitHub para baixar e atualizar
os programas, pelo yt-dlp aos sites dos links que você der e, para a
miniatura na janela, ao endereço da imagem que o yt-dlp informa. Nada além
disso. A [política de privacidade](#política-de-privacidade) tem os detalhes.

**Com quais sites ele funciona?**
Com os [sites suportados pelo yt-dlp](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md),
desde que o vídeo não peça login.

**Onde ficam os meus vídeos?**
Na pasta de downloads que você escolheu na primeira execução, que aparece na
barra de baixo da janela. **Open folder**, com o arquivo salvo, abre essa
pasta; **Change** escolhe outra.

**Um download falha num site que funcionava antes. E agora?**
O site provavelmente mudou, e o yt-dlp pode precisar de uma versão mais nova.
O keepclip procura uma versão nova toda semana, e as configurações (a
engrenagem) mostram quando foi a última vez. Copie o motivo que o keepclip
mostra e procure nas [issues do yt-dlp](https://github.com/yt-dlp/yt-dlp/issues). Se o
keepclip mostrar o problema de um jeito que o yt-dlp sozinho não mostra,
[abra uma issue](https://github.com/iglesiaskun/keepclip/issues/new/choose)
aqui.

**Por que o keepclip baixa também o ffmpeg e o Deno?**
O yt-dlp precisa deles: o ffmpeg junta o vídeo e o som, que muitos sites
guardam separados, e gera os MP3; o Deno roda o JavaScript que alguns sites
pedem. Eles vêm dos próprios projetos, com as próprias licenças, então o
keepclip os baixa em vez de trazê-los junto.

**Como volto para a configuração padrão?**
Apague o `%APPDATA%\keepclip\config.toml`. Da próxima vez que abrir, o
keepclip pergunta de novo a pasta de downloads.

**Por que "keepclip"?**
Ele guarda (*keep*) o clipe que você achou: uma cópia do vídeo, na sua
própria pasta.

## Próximos passos

Esta primeira versão é pequena de propósito, e o que as pessoas disserem dela
vai guiar as próximas. Na lista, por enquanto:

- Versões com assinatura digital, para o SmartScreen parar de avisar (veja a
  [política de assinatura de código](#política-de-assinatura-de-código)).
- Vídeos que pedem login.
- Nomes de arquivo mais curtos para títulos cheios de hashtags e emoji.
- Atualizar o ffmpeg e o Deno de tempos em tempos, como o keepclip já faz com
  o yt-dlp.

Ideias são bem-vindas como
[pedidos de recurso](https://github.com/iglesiaskun/keepclip/issues/new/choose).

## Como contribuir

Relatos de bugs, ideias e pull requests são bem-vindos. O
[CONTRIBUTING.md](CONTRIBUTING.md) (em inglês) explica como as mudanças
entram e como compilar o keepclip a partir do código.

## Segurança

Problemas de segurança devem ser relatados em particular, como o
[SECURITY.md](SECURITY.md) explica.

## Política de assinatura de código

O pedido de assinatura de código gratuita à
[SignPath Foundation](https://signpath.org), que assina projetos open source,
está em andamento. Até ele ser aprovado, os arquivos das versões não têm
assinatura; ["Limitações conhecidas"](#limitações-conhecidas) explica como
conferi-los enquanto isso.

Depois de aprovado, cada versão é assinada assim:

- Só vão para a assinatura os arquivos que o
  [workflow de release](.github/workflows/release.yml) compila no GitHub
  Actions, a partir de uma tag de versão deste repositório: o `keepclip.exe`
  e o `keepclip-cli.exe`, que vão no `.msi` e no `.zip`, e depois o próprio
  `.msi`.
- Um aprovador confere e aprova à mão cada pedido de assinatura.
- Os programas que o keepclip baixa na primeira execução (yt-dlp, ffmpeg,
  Deno) não fazem parte do keepclip e não são assinados por ele. No lugar
  disso, o keepclip confere cada um com o SHA-256 publicado junto.

### Papéis

- Autores, que podem mudar o código sem outra revisão:
  Pedro ([@iglesiaskun](https://github.com/iglesiaskun))
- Revisores, que conferem toda mudança de quem não é autor:
  Pedro ([@iglesiaskun](https://github.com/iglesiaskun))
- Aprovadores, que decidem se cada pedido de assinatura é assinado:
  Pedro ([@iglesiaskun](https://github.com/iglesiaskun))

Toda mudança, inclusive as do mantenedor, chega ao `main` por um pull request
que passa no CI.

### Política de privacidade

O keepclip não manda telemetria, não pede conta e não procura atualizações de
si mesmo. Ele só se conecta a outros computadores para fazer o que você pede:

- **O site do link.** O yt-dlp lê o link que você der e baixa o vídeo desse
  site e dos servidores que o site indicar, às vezes pelo ffmpeg. Isso fica
  entre você e o site, sob os termos e a política de privacidade dele.
- **Os programas que ele baixa.** Na primeira execução, depois que você
  concorda, o keepclip baixa o yt-dlp, o ffmpeg e o Deno, com as somas de
  verificação, das releases oficiais de cada um no GitHub.
- **Atualizações do yt-dlp.** Antes de ler um link, quando a última checagem
  tem mais de 7 dias, o keepclip roda `yt-dlp -U`, que procura um yt-dlp mais
  novo no GitHub e o instala.
- **Miniaturas.** A janela mostra uma miniatura de cada vídeo, que o keepclip
  baixa do endereço informado pelo yt-dlp, em geral nos servidores do site.

O Deno roda o JavaScript do site no seu computador, para o yt-dlp, sem acesso
à rede. Como em qualquer download, cada servidor vê o seu endereço IP. As
requisições do próprio keepclip não levam nada sobre você: elas se
identificam só como `keepclip/<versão>`. Os downloads do GitHub seguem a
[Declaração de Privacidade Geral do GitHub](https://docs.github.com/pt/site-policy/privacy-policies/github-general-privacy-statement).

## Licença

O keepclip é distribuído sob a [licença MIT](LICENSE-MIT) ou a
[Apache License, Version 2.0](LICENSE-APACHE), à sua escolha.

Os programas que o keepclip baixa na primeira execução (yt-dlp, ffmpeg, Deno)
não fazem parte do keepclip e mantêm as próprias licenças.

Os ícones da janela vêm do [Lucide](https://lucide.dev) e estão sob a licença
ISC ([LICENSE-LUCIDE](window/ui/icons/LICENSE-LUCIDE)).

## Referência do arquivo de configuração

O arquivo de configuração fica em `%APPDATA%\keepclip\config.toml`. O
keepclip o escreve quando você escolhe a pasta de downloads (na primeira
execução, com **Change** na barra de baixo da janela ou nas configurações, ou
com `--choose-folder`), o lê em toda execução, e você pode editá-lo à mão. Se
o keepclip não conseguir ler o arquivo, ele diz por quê, com o caminho do
arquivo, e não abre até o arquivo ser corrigido ou apagado.

```toml
# keepclip settings. You can edit this file; keepclip reads it on every run.
version = 1

# Folder where downloads are saved. `keepclip --choose-folder` changes it.
download_dir = 'C:\Users\voce\Videos'
```

A linha `version = 1` é o formato do arquivo. Se uma versão futura mudar o
formato, o keepclip usa esse número para atualizar arquivos antigos. Um
arquivo sem ele conta como versão 1.

**Linha de comando.** O `keepclip` e o `keepclip-cli` aceitam as mesmas
opções, num terminal ou num atalho:

| Comando | Faz |
|---|---|
| `keepclip` | abre com o link copiado no campo, pronto para ler |
| `keepclip --clipboard` | lê na hora o link da área de transferência |
| `keepclip <link>` | lê esse link na hora |
| `keepclip --choose-folder` | escolhe de novo a pasta de downloads; sozinho, abre as configurações |
| `keepclip --dir <pasta>` | guarda o arquivo de configuração nessa pasta, útil para testes |
| `keepclip --version` | mostra a versão |
| `keepclip --help` | mostra as opções |

Sem link, o `keepclip-cli` lê na hora o da área de transferência, ou pede um.
