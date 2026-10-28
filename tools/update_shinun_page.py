# מעדכן את לשונית האבחנה המבדלת בדף "שינון לשלב ב'" לפי ddx-data.json
# שימוש: python3 tools/update_shinun_page.py <קובץ הדף הנוכחי> ddx-data.json <קובץ פלט>
import json, re, sys, html
src, data_p, out_p = sys.argv[1:4]
s = open(src, encoding='utf-8').read()
D = json.load(open(data_p, encoding='utf-8'))
a = s.index('<section class="sec dxsec" id="dx1">'); b = s.index('<p class="empty" id="empty-ddx"')
old = s[a:b]
orig = {html.unescape(m.group(1)): m.group(0) for m in re.finditer(r'<article class="card dx" data-name="([^"]*)">.*?</article>', old, re.S)}
sec_ids = dict(re.findall(r'<section class="sec dxsec" id="(dx\d)"><h2>(.*?)</h2>', old))
sys_to_id = {html.unescape(v): k for k, v in sec_ids.items()}

def esc(t): return html.escape(t, quote=False).replace('"', '&quot;')
def bidi(t):
    # מפצלים לפני ההמרה לקוד HTML, כדי לא לשבור את &quot;
    parts = re.split(r'([A-Za-z][A-Za-z0-9.+\-/ ]*[A-Za-z0-9+]|[A-Za-z])', t)
    return ''.join(f'<bdi dir="ltr">{esc(p)}</bdi>' if i % 2 else esc(p) for i, p in enumerate(parts))
def gen(d):
    h = f'<article class="card dx" data-name="{esc(d["name"])}"><header><h3>{bidi(d["name"])}</h3></header>'
    for blk in d['b']:
        if blk['t'] == 'sub': h += f'<div class="cut-t dxsub">{bidi(blk["v"])}</div>'
        elif blk['t'] == 'note': h += f'<p class="note">{bidi(blk["v"])}</p>'
        elif blk['t'] == 'rows': h += '<ul class="dxrows">' + ''.join(f'<li><span class="grp">{bidi(g)}</span><span class="dxs ans">{bidi(x)}</span></li>' for g, x in blk['v']) + '</ul>'
        elif blk['t'] == 'red': h += f'<p class="trap"><b>{bidi(blk["l"])}:</b> <span class="ans">{bidi(blk["v"])}</span></p>'
    return h + '</article>'

def text(h): return re.sub(r'\s+', ' ', html.unescape(re.sub(r'<[^>]+>', '', h))).strip()

renamed = {'ירידה במשקל': 'ירידה לא מכוונת במשקל'}
kept = gen_n = 0
parts = []
systems = []
for d in D:
    if d['sys'] not in systems: systems.append(d['sys'])
for sname in systems:
    cards = []
    for d in [x for x in D if x['sys'] == sname]:
        on = renamed.get(d['name'], d['name'])
        # כרטיס קיים נשמר כמו שהוא (כולל עיצוב מקורי) רק אם הטקסט שלו זהה למאגר; אחרת נבנה מחדש
        broken = re.search(r'&<bdi dir="ltr">[a-z]+</bdi>;', orig.get(on, ''))   # תווים שנשברו בגרסה קודמת של הכלי
        if on in orig and not broken and text(orig[on]).replace(on, d['name'], 1) == text(gen(d)):
            c = orig[on]
            if on != d['name']:
                c = c.replace(f'data-name="{esc(on)}"', f'data-name="{esc(d["name"])}"').replace(f'<h3>{esc(on)}</h3>', f'<h3>{esc(d["name"])}</h3>')
            cards.append(c); kept += 1
        else:
            cards.append(gen(d)); gen_n += 1
    parts.append(f'<section class="sec dxsec" id="{sys_to_id[sname]}"><h2>{esc(sname)}</h2><div class="cards wide">' + ''.join(cards) + '</div></section>')
new = '\n      '.join(parts) + '\n      '
s = s[:a] + new + s[b:]
# שורת הכותרת של הלשונית: מספר התסמינים ותאריך העדכון
import datetime
s = re.sub(r"meta: '\d+ תסמינים · עודכן [^']*'", f"meta: '{len(D)} תסמינים · עודכן {datetime.date.today():%d.%m.%Y}'", s)
open(out_p, 'w', encoding='utf-8').write(s)
print('kept', kept, 'generated', gen_n, 'total', kept + gen_n)
