# בונה את ddx.html מתבנית האפליקציה ומקובץ הנתונים
import json
data = json.load(open('ddx-data.json', encoding='utf-8'))
src = open('ddx.src.html', encoding='utf-8').read()
blob = json.dumps(data, ensure_ascii=False, separators=(',', ':')).replace('</', '<\\/')
open('ddx.html', 'w', encoding='utf-8').write(src.replace('/*DATA*/', blob))
print('ddx.html:', len(data), 'symptoms')
