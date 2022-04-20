#!/bin/sh

# Convert ANSI (terminal) colours and attributes to HTML

# Author:
#    http://www.pixelbeat.org/docs/terminal_colours/
# Examples:
#    ls -l --color=always | ansi2html.sh > ls.html
#    git show --color | ansi2html.sh > last_change.html
#    Generally one can use the `script` util to capture full terminal output.
# Changes:
#    V0.1, 24 Apr 2008, Initial release
#    V0.2, 01 Jan 2009, Phil Harnish <philharnish@gmail.com>
#                         Support `git diff --color` output by
#                         matching ANSI codes that specify only
#                         bold or background colour.
#                       P@draigBrady.com
#                         Support `ls --color` output by stripping
#                         redundant leading 0s from ANSI codes.
#                         Support `grep --color=always` by stripping
#                         unhandled ANSI codes (specifically ^[[K).
#    V0.3, 20 Mar 2009, http://eexpress.blog.ubuntu.org.cn/
#                         Remove cat -v usage which mangled non ascii input.
#                         Cleanup regular expressions used.
#                         Support other attributes like reverse, ...
#                       P@draigBrady.com
#                         Correctly nest <span> tags (even across lines).
#                         Add a command line option to use a dark background.
#                         Strip more terminal control codes.
#    V0.4, 17 Sep 2009, P@draigBrady.com
#                         Handle codes with combined attributes and color.
#                         Handle isolated <bold> attributes with css.
#                         Strip more terminal control codes.
#    V0.12, 12 Jul 2011
#      http://github.com/pixelb/scripts/commits/master/scripts/ansi2html.sh

if [ "$1" = "--version" ]; then
    echo "0.12" && exit
fi

if [ "$1" = "--help" ]; then
    echo "This utility converts ANSI codes in data passed to stdin" >&2
    echo "It has 2 optional parameters:" >&2
    echo "   --bg=dark --palette=linux|solarized|tango|xterm" >&2
    echo "E.g.: ls -l --color=always | ansi2html.sh --bg=dark > ls.html" >&2
    exit
fi

# dark_bg=false
# [ "$1" = "--bg=dark" ] && { dark_bg=true; shift; }

if [ "$1" = "--palette=solarized" ]; then
   # See http://ethanschoonover.com/solarized
   P0=073642;  P1=D30102;  P2=859900;  P3=B58900;
   P4=268BD2;  P5=D33682;  P6=2AA198;  P7=EEE8D5;
   P8=002B36;  P9=CB4B16; P10=586E75; P11=657B83;
  P12=839496; P13=6C71C4; P14=93A1A1; P15=FDF6E3;
  shift;
elif [ "$1" = "--palette=solarized-xterm" ]; then
   # Above mapped onto the xterm 256 color palette
   P0=262626;  P1=AF0000;  P2=5F8700;  P3=AF8700;
   P4=0087FF;  P5=AF005F;  P6=00AFAF;  P7=E4E4E4;
   P8=1C1C1C;  P9=D75F00; P10=585858; P11=626262;
  P12=808080; P13=5F5FAF; P14=8A8A8A; P15=FFFFD7;
  shift;
elif [ "$1" = "--palette=tango" ]; then
   # Gnome default
   P0=000000;  P1=CC0000;  P2=4E9A06;  P3=C4A000;
   P4=3465A4;  P5=75507B;  P6=06989A;  P7=D3D7CF;
   P8=555753;  P9=EF2929; P10=8AE234; P11=FCE94F;
  P12=729FCF; P13=AD7FA8; P14=34E2E2; P15=EEEEEC;
  shift;
elif [ "$1" = "--palette=xterm" ]; then
   P0=000000;  P1=CD0000;  P2=00CD00;  P3=CDCD00;
   P4=0000EE;  P5=CD00CD;  P6=00CDCD;  P7=E5E5E5;
   P8=7F7F7F;  P9=FF0000; P10=00FF00; P11=FFFF00;
  P12=5C5CFF; P13=FF00FF; P14=00FFFF; P15=FFFFFF;
  shift;
else # linux console
   P0=000000;  P1=AA0000;  P2=00AA00;  P3=AA5500;
   P4=0000AA;  P5=AA00AA;  P6=00AAAA;  P7=AAAAAA;
   P8=555555;  P9=FF5555; P10=55FF55; P11=FFFF55;
  P12=5555FF; P13=FF55FF; P14=55FFFF; P15=FFFFFF;
  [ "$1" = "--palette=linux" ] && shift
fi

   # D0=262626;  D1=AF0000;  D2=5F8700;  D3=AF8700;
   # D4=0087FF;  D5=AF005F;  D6=00AFAF;  D7=E4E4E4;
   # D8=1C1C1C;  D9=D75F00; D10=585858; D11=626262;
  # D12=808080; D13=5F5FAF; D14=8A8A8A; D15=FFFFD7;
   # D0=000000;  D1=CD0000;  D2=00CD00;  D3=CDCD00;
   # D4=0000EE;  D5=CD00CD;  D6=00CDCD;  D7=E5E5E5;
   # D8=7F7F7F;  D9=FF0000; D10=00FF00; D11=FFFF00;
  # D12=5C5CFF; D13=FF00FF; D14=00FFFF; D15=FFFFFF;
bg=2c2c2c

# breeze from https://github.com/eendroroy/alacritty-theme/blob/master/schemes.yaml
bg=232627
fg=fcfcfc
dim_fg=63686d # eff0f1
bright_fg=ffffff
dim_bg=0x31363b
bright_bg=000000

# normal:
P0='232627' # black
P1='ed1515' # red
P2='11d116' # green
P3='f67400' # yellow
P4='1d99f3' # blue
P5='9b59b6' # magenta
P6='1abc9c' # cyan
P7='fcfcfc' # white
# bright:
P8='7f8c8d' # black
P9='c0392b' # red
P10='1cdc9a' # green
P11='fdbc4b' # yellow
P12='3daee9' # blue
P13='8e44ad' # magenta
P14='16a085' # cyan
P15='ffffff' # white
# dim:
P16='81868b' # black
P17='783228' # red
P18='17a262' # green
P19='b65619' # yellow
P20='1b668f' # blue
P21='614a73' # magenta
P22='186c60' # cyan
P23='63686d' # white


# P16=$(printf "%2.2x%2.2x%2.2x" 152 152 152)
# P17=$(printf "%2.2x%2.2x%2.2x" 152 122 122)
# P18=$(printf "%2.2x%2.2x%2.2x" 87 163 124)
# P19=$(printf "%2.2x%2.2x%2.2x" 170 133 111)
# P20=$(printf "%2.2x%2.2x%2.2x" 117 141 161)
# P21=$(printf "%2.2x%2.2x%2.2x" 154 98 137)
# P22=$(printf "%2.2x%2.2x%2.2x" 107 159 161)
# P23=$(printf "%2.2x%2.2x%2.2x" 149 149 139)

# [ "$1" = "--bg=dark" ] && { dark_bg=true; shift; }

# if $dark_bg; then
#   temp=$P0
#   P0=$P7
#   P7=$temp
#   temp=$P8
#   P8=$P15
#   P15=$temp
#   temp=$P16
#   P16=$P23
#   P23=$temp
# fi

# dark_bg=false

echo "<html>
<head>
<meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/>
<style type=\"text/css\">
body {
  color: #$fg;
  background-color: #$bg;
}
.dim { color: #$dim_fg; }

.f0 { color: #$P0; } .b0 { background-color: #$P0; }
.f1 { color: #$P1; } .b1 { background-color: #$P1; }
.f2 { color: #$P2; } .b2 { background-color: #$P2; }
.f3 { color: #$P3; } .b3 { background-color: #$P3; }
.f4 { color: #$P4; } .b4 { background-color: #$P4; }
.f5 { color: #$P5; } .b5 { background-color: #$P5; }
.f6 { color: #$P6; } .b6 { background-color: #$P6; }
.f7 { color: #$P7; } .b7 { background-color: #$P7; }
.f0 .bold, .bold .f0 { color: #$P8; }
.f1 .bold, .bold .f1 { color: #$P9; }
.f2 .bold, .bold .f2 { color: #$P10; }
.f3 .bold, .bold .f3 { color: #$P11; }
.f4 .bold, .bold .f4 { color: #$P12; }
.f5 .bold, .bold .f5 { color: #$P13; }
.f6 .bold, .bold .f6 { color: #$P14; }
.f7 .bold, .bold .f7 { color: #$P15; }
.f0 .dim, .dim .f0 { color: #$P16; }
.f1 .dim, .dim .f1 { color: #$P17; }
.f2 .dim, .dim .f2 { color: #$P18; }
.f3 .dim, .dim .f3 { color: #$P19; }
.f4 .dim, .dim .f4 { color: #$P20; }
.f5 .dim, .dim .f5 { color: #$P21; }
.f6 .dim, .dim .f6 { color: #$P22; }
.f7 .dim, .dim .f7 { color: #$P23; }
.bold { font-weight: bold; }
// .dim { opacity: 0.5; font-weight: lighter; }
.fast-blink { animation: blinker 0.2s linear infinite; }
.blink { animation: blinker 1s linear infinite; }
@keyframes blinker { 50% { opacity: 0.2; } }
.eb8  { background-color: #$P8; }
.eb9  { background-color: #$P9; }
.eb10 { background-color: #$P10; }
.eb11 { background-color: #$P11; }
.eb12 { background-color: #$P12; }
.eb13 { background-color: #$P13; }
.eb14 { background-color: #$P14; }
.eb15 { background-color: #$P15; }
"
# .ef8, .f0 > .bold,.bold > .f0 { color: #$P8; font-weight: bold; }
# .ef9, .f1 > .bold,.bold > .f1 { color: #$P9; font-weight: bold; }
# .ef10,.f2 > .bold,.bold > .f2 { color: #$P10; font-weight: bold; }
# .ef11,.f3 > .bold,.bold > .f3 { color: #$P11; font-weight: bold; }
# .ef12,.f4 > .bold,.bold > .f4 { color: #$P12; font-weight: bold; }
# .ef13,.f5 > .bold,.bold > .f5 { color: #$P13; font-weight: bold; }
# .ef14,.f6 > .bold,.bold > .f6 { color: #$P14; font-weight: bold; }
# .ef15,.f7 > .bold,.bold > .f7 { color: #$P15; font-weight: bold; }

# .dim { color: #$P23; font-weight: light; }

# The default xterm 256 colour palette
# for red in $(seq 0 5); do
#   for green in $(seq 0 5); do
#     for blue in $(seq 0 5); do
#         c=$((16 + ($red * 36) + ($green * 6) + $blue))
#         r=$((($red * 40 + 55) * ($red > 0)))
#         g=$((($green * 40 + 55) * ($green > 0)))
#         b=$((($blue * 40 + 55) * ($blue > 0)))
#         printf ".ef%d { color: #%2.2x%2.2x%2.2x; } " $c $r $g $b
#         printf ".eb%d { background-color: #%2.2x%2.2x%2.2x; }\n" $c $r $g $b
#     done
#   done
# done
# for gray in $(seq 0 23); do
#   c=$(($gray+232))
#   l=$(($gray*10 + 8))
#   printf ".ef%d { color: #%2.2x%2.2x%2.2x; } " $c $l $l $l
#   printf ".eb%d { background-color: #%2.2x%2.2x%2.2x; }\n" $c $l $l $l
# done

# .f9 { color: #$($dark_bg && echo $P7 || echo $P0); }
# .b9 { background-color: #$($dark_bg && echo $P7 || echo $P15); }
  # color: #$($dark_bg && echo $P15 || echo $P0);
  # font-weight: $($dark_bg && echo normal || echo bold);
  # color: #$($dark_bg && echo $P8 || echo $P0);
  # font-weight: $($dark_bg && echo normal || echo bold);
  # font-weight: bold;
# .f9 { color: #$fg; }
# .b9 { background-color: #$bg; }
cat << EOF
.f9 > .bold,.bold > .f9, body.f9 > pre > .bold {
  /* Bold is heavy black on white, or bright white
     depending on the default background */
  color: #$bg;
}
.reverse {
  /* CSS doesnt support swapping fg and bg colours unfortunately,
     so just hardcode something that will look OK on all backgrounds. */
  '"color: #$P0; background-color: #$P7;"'
}
.underline { text-decoration: underline; }
.line-through { text-decoration: line-through; }
.blink { text-decoration: blink; }
</style>
</head>
<body class="f9 b9">
<pre>
EOF

p='\x1b\['        #shortcut to match escape codes
P="\(^[^°]*\)¡$p" #expression to match prepended codes below

# Handle various xterm control sequences.
# See /usr/share/doc/xterm-*/ctlseqs.txt
sed "
s#\x1b[^\x1b]*\x1b\\\##g  # strip anything between \e and ST
s#\x1b][0-9]*;[^\a]*\a##g # strip any OSC (xterm title etc.)
#handle carriage returns
s#^.*\r\{1,\}\([^$]\)#\1#
s#\r\$## # strip trailing \r
# strip other non SGR escape sequences
s#[\x07]##g
s#\x1b[]>=\][0-9;]*##g
s#\x1bP+.\{5\}##g
s#${p}[0-9;?]*[^0-9;?m]##g
#remove backspace chars and what they're backspacing over
:rm_bs
s#[^\x08]\x08##g; t rm_bs
" |

# Normalize the input before transformation
sed "
# escape HTML
s#\&#\&amp;#g; s#>#\&gt;#g; s#<#\&lt;#g; s#\"#\&quot;#g
# normalize SGR codes a little
# split 256 colors out and mark so that they're not
# recognised by the following 'split combined' line
:e
s#${p}\([0-9;]\{1,\}\);\([34]8;5;[0-9]\{1,3\}\)m#${p}\1m${p}¬\2m#g; t e
s#${p}\([34]8;5;[0-9]\{1,3\}\)m#${p}¬\1m#g;
:c
s#${p}\([0-9]\{1,\}\);\([0-9;]\{1,\}\)m#${p}\1m${p}\2m#g; t c   # split combined
s#${p}0\([0-7]\)#${p}\1#g                                 #strip leading 0
s#${p}1m\(\(${p}[4579]m\)*\)#\1${p}1m#g                   #bold last (with clr)
s#${p}2m\(\(${p}[4579]m\)*\)#\1${p}2m#g                   #dim last (with clr)
s#${p}m#${p}0m#g                                          #add leading 0 to norm
# undo any 256 color marking
s#${p}¬\([34]8;5;[0-9]\{1,3\}\)m#${p}\1m#g;
# map 16 color codes to color + bold
s#${p}9\([0-7]\)m#${p}3\1m${p}1m#g;
s#${p}10\([0-7]\)m#${p}4\1m${p}1m#g;
# change 'reset' code to a single char, and prepend a single char to
# other codes so that we can easily do negative matching, as sed
# does not support look behind expressions etc.
s#°#\&deg;#g; s#${p}0m#°#g
s#¡#\&iexcl;#g; s#${p}[0-9;]*m#¡&#g
" |

# Convert SGR sequences to HTML
sed "
:ansi_to_span # replace ANSI codes with CSS classes
t ansi_to_span # hack so t commands below only apply to preceeding s cmd
/^[^¡]*°/ { b span_end } # replace 'reset code' if no preceeding code
# common combinations to minimise html (optional)
s#${P}3\([0-7]\)m¡${p}4\([0-7]\)m#\1<span class=\"f\2 b\3\">#;t span_count
s#${P}4\([0-7]\)m¡${p}3\([0-7]\)m#\1<span class=\"f\3 b\2\">#;t span_count
s#${P}1m#\1<span class=\"bold\">#;                            t span_count
s#${P}2m#\1<span class=\"dim\">#;                             t span_count
s#${P}4m#\1<span class=\"underline\">#;                       t span_count
s#${P}5m#\1<span class=\"blink\">#;                           t span_count
s#${P}6m#\1<span class=\"fast-blink\">#;                      t span_count
s#${P}7m#\1<span class=\"reverse\">#;                         t span_count
s#${P}9m#\1<span class=\"line-through\">#;                    t span_count
s#${P}3\([0-9]\)m#\1<span class=\"f\2\">#;                    t span_count
s#${P}4\([0-9]\)m#\1<span class=\"b\2\">#;                    t span_count
# s#${P}38;5;\([0-9]\{1,3\}\)m#\1<span class=\"ef\2\">#;        t span_count
s#${P}38;5;\([0-9]\{1,3\}\)m#\1<span class=\"f\2\">#;        t span_count
s#${P}48;5;\([0-9]\{1,3\}\)m#\1<span class=\"b\2\">#;        t span_count
s#${P}[0-9;]*m#\1#g; t ansi_to_span # strip unhandled codes
b # next line of input
# add a corresponding span end flag
:span_count
x; s/^/s/; x
b ansi_to_span
# replace 'reset code' with correct number of </span> tags
:span_end
x
/^s/ {
  s/^.//
  x
  s#°#</span>°#
  b span_end
}
x
s#°##
b ansi_to_span
" |

# Convert alternative character set
# Note we convert here, as if we do at start we have to worry about avoiding
# conversion of SGR codes etc., whereas doing here we only have to
# avoid conversions of stuff between &...; or <...>
#
# Note we could use sed to do this based around:
#   sed 'y/abcdefghijklmnopqrstuvwxyz{}`~/▒␉␌␍␊°±␤␋┘┐┌└┼⎺⎻─⎼⎽├┤┴┬│≤≥π£◆·/'
# However that would be very awkward as we need to only conv some input.
# The basic scheme that we do in the python script below is:
#  1. enable transliterate once ¡ char seen
#  2. disable once µ char seen (may be on diff line to ¡)
#  3. never transliterate between &; or <> chars
sed "
# change 'smacs' and 'rmacs' to a single char so that we can easily do
# negative matching, as sed does not support look behind expressions etc.
# Note we don't use ° like above as that's part of the alternate charset.
s#\x1b(0#¡#g;
s#µ#\&micro;#g; s#\x1b(B#µ#g
" |
(
python -c "
# vim:fileencoding=utf8
import sys
import locale
encoding=locale.getpreferredencoding()
old='abcdefghijklmnopqrstuvwxyz{}\`~'
new='▒␉␌␍␊°±␤␋┘┐┌└┼⎺⎻─⎼⎽├┤┴┬│≤≥π£◆·'
new=unicode(new, 'utf-8')
table=range(128)
for o,n in zip(old, new): table[ord(o)]=n
(STANDARD, ALTERNATIVE, HTML_TAG, HTML_ENTITY) = (0, 1, 2, 3)
state = STANDARD
last_mode = STANDARD
for c in unicode(sys.stdin.read(), encoding):
  if state == HTML_TAG:
    if c == '>':
      state = last_mode
  elif state == HTML_ENTITY:
    if c == ';':
      state = last_mode
  else:
    if c == '<':
      state = HTML_TAG
    elif c == '&':
      state = HTML_ENTITY
    elif c == u'¡' and state == STANDARD:
      state = ALTERNATIVE
      last_mode = ALTERNATIVE
      continue
    elif c == u'µ' and state == ALTERNATIVE:
      state = STANDARD
      last_mode = STANDARD
      continue
    elif state == ALTERNATIVE:
      c = c.translate(table)
  sys.stdout.write(c.encode(encoding))
" 2>/dev/null ||
sed 's/[¡µ]//g' # just strip aternative flag chars
)

echo "</pre>
</body>
</html>"
