"""Import source worksheets and speech cards for offline use."""
import json
import sys
from pathlib import Path
import pypdfium2 as pdfium

DEST = Path(__file__).resolve().parents[1] / "assets" / "practice"
BANDS = [
 [(144,267),(285,412),(431,569),(588,719),(739,880),(899,1031),(1050,1192)],
 [(147,272),(291,420),(440,580),(600,743),(763,899),(918,1048),(1067,1196)],
 [(146,267),(284,404),(420,541),(558,677),(694,814),(832,950),(967,1087),(1103,1214)],
 [(145,269),(286,413),(430,560),(577,702),(720,842),(858,973),(988,1093),(1109,1208)],
 [(125,251),(269,401),(419,551),(568,699),(717,847),(865,993),(1009,1114),(1130,1225)],
 [(120,247),(265,396),(412,546),(563,691),(709,836),(852,972),(988,1099),(1114,1224)],
 [(128,254),(271,399),(417,547),(564,692),(709,837),(854,979),(995,1115),(1132,1225)],
 [(125,254),(271,403),(421,553),(569,700),(717,845),(861,984),(1000,1114),(1130,1225)],
]
SPEECH = [
 [('baby','Baby','बच्चा','BA'),('dad','Dad','पापा','DA'),('mom','Mom','माँ','MA'),('panda','Panda','पांडा','PA')],
 [('lamp','Lamp','लैंप','AM'),('fan','Fan','पंखा','AN'),('apple','Apple','सेब','AP'),('cat','Cat','बिल्ली','AT')],
 [('ball','Ball','गेंद',''),('doll','Doll','गुड़िया',''),('blocks','Blocks','ब्लॉक',''),('train','Train','रेलगाड़ी','')],
 [('hat','Hat','टोपी',''),('shoes','Shoes','जूते',''),('socks','Socks','मोज़े',''),('shirt','Shirt','कमीज़','')],
 [('chair','Chair','कुर्सी',''),('table','Table','मेज़',''),('bed','Bed','बिस्तर',''),('book','Book','किताब','')],
 [('happy','Happy','खुश',''),('sad','Sad','उदास',''),('hungry','Hungry','भूखा',''),('tired','Tired','थका हुआ','')],
]

def run(source):
    DEST.mkdir(parents=True, exist_ok=True)
    doc = pdfium.PdfDocument(source / 'combined_pencil_control.pdf')
    assert len(doc) == 14
    manifest = []
    for i in range(14):
        image = doc[i].render(scale=1.5).to_pil().convert('RGB')
        image.save(DEST / f'pencil-{i+1}.jpg', quality=90)
        if i < 8:
            for row, (top,bottom) in enumerate(BANDS[i]):
                crop = image.crop((43,top-6,856,min(image.height,bottom+6)))
                file = f'pencil-{i+1}-row-{row+1}.jpg'
                crop.save(DEST / file, quality=93)
                manifest.append(dict(page=i+1,row=row+1,file=file))
        else:
            manifest.append(dict(page=i+1,row=0,file=f'pencil-{i+1}.jpg'))
    doc = pdfium.PdfDocument(source / 'Speech-Practice-Flashcards.pdf')
    assert len(doc) == 7
    cards = []
    for i, items in enumerate(SPEECH):
        image = doc[i+1].render(scale=2).to_pil().convert('RGB')
        w,h = image.size
        for j,(id,en,hi,sound) in enumerate(items):
            left,right = (.100,.452) if j%2==0 else (.555,.907)
            top,bottom = ((.190,.501) if j<2 else (.556,.870)) if i<5 else ((.173,.485) if j<2 else (.539,.851))
            image.crop((round(w*left),round(h*top),round(w*right),round(h*bottom))).save(DEST/f'speech-{id}.jpg',quality=94)
            cards.append(dict(id='speech-'+id,en=en,hi=hi,sound=sound,page=i+2,source='Autism Parenting Magazine, Speech Practice Flashcards'))
    (DEST/'pencil.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
    (DEST/'speech.json').write_text(json.dumps(cards,ensure_ascii=False,indent=2),encoding='utf-8')
    print(f'Imported {len(manifest)} tracing activities and {len(cards)} speech cards')

if __name__ == '__main__':
    run(Path(sys.argv[1]))
