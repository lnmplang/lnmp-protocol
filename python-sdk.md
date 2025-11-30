1. Stratejik Karar: Rust Tek Gerçek Kaynak, Python İnce Katman

Hedefimiz:
Python SDK’yı, Rust tarafında zaten var olan LNMP ekosisteminin ince bir adapter’i olarak tasarlamak.

Protokol, determinism, codec, checksum, envelope, net, routing, SFE vs:

Tek canonical implementation: Rust (lnmp meta crate)

Python dahil tüm SDK’lar bunu kullanacak.

Python tarafında:

Hiçbir kritik algoritmayı yeniden yazmıyoruz.

Sadece Rust fonksiyonlarını Python’dan doğal hissettiren bir API ile açıyoruz.

Bu sayede:

Tek yerde (Rust) bug fix ve feature geliştirme,

Python/TS/Go tarafında sadece “yüzey” değişikliği,

Sürüm yönetimi ve davranış tutarlılığı çok daha kolay.

2. Mevcut Durum: Rust Meta Crate Avantajı

Şu anda:

lnmp meta crate’i:

lnmp-core

lnmp-codec

lnmp-envelope

lnmp-net

lnmp-sfe

lnmp-transport

lnmp-sanitize

lnmp-embedding

lnmp-spatial

lnmp-quant

lnmp-llb

… hepsini tek yerden re-export ediyor.

Python tarafındaki binding’ler için bu büyük avantaj:

Binding crate → sadece lnmp meta crate’e depend edecek,

Her modülü tek tek wired etmek zorunda kalmayacağız,

Version skew/uyumsuzluk riski düşecek.

3. Python SDK Mimarisi: 2 Katman
3.1. Katman 1 – Rust Binding (low-level, pyo3)

Crate adı önerisi: lnmp-sdk-python
Konum: bindings/python/lnmp-sdk-python (veya benzeri)

Bu crate:

Workspace’in bir üyesi olacak.

lnmp meta crate’e depend edecek:

Tüm core behaviour oradan gelecek.

pyo3 ile Python’a native extension modülü olarak expose edilecek.

Bu modül Python’da kabaca şöyle görünecek (konsept olarak):

Tipler (class):

PyLnmpRecord – içerde LnmpRecord

PyLnmpEnvelope – içerde LnmpEnvelope

Gerekirse PyLnmpNetMessage (veya sadece envelope+metadata)

Fonksiyonlar:

parse(text: str) -> PyLnmpRecord

encode(record: PyLnmpRecord) -> str

encode_binary(record: PyLnmpRecord) -> bytes

decode_binary(data: bytes) -> PyLnmpRecord

envelope_wrap(record, source, timestamp_ms?, trace_id?) -> PyLnmpEnvelope

routing_decide(envelope) -> str
("SendToLLM" | "Drop" | "ProcessLocally" gibi)

context_score(envelope) -> { composite, freshness, importance, risk }

İleride: embedding delta, spatial encode vs. gerekirse eklenir.

Teknik tercihler:

pyo3:

Rust struct’ları Python class’ı gibi expose etmek için,

Hata handling: Rust error → Python exception mapping.

maturin:

Wheel üretmek için (Linux/Mac/Windows, CPython 3.9+),

PyPI yayın pipeline’ını sadeleştirmek için.

Bu katman çok “Rust kokacak”, ama Python’a minimum primitives’i açmamız için gerekli.

3.2. Katman 2 – Pythonic API (high-level wrapper)

Python paketi adı önerisi: lnmp (veya lnmp_py)
Konum: bindings/python/lnmp/

Bu paket:

Kullanıcıya “Rust detayını unutturan” layer olacak.

lnmp_core_py (binding modülü) üstünden çağrı yapacak.

Modül yapısı örneği:

lnmp/
  __init__.py
  core.py        # parse/encode; Record wrapper
  envelope.py    # envelope_wrap; Envelope wrapper
  net.py         # kind/priority/ttl/class helper'ları
  llm.py         # normalize + route + ShortForm kullanım senaryoları


Yüksek seviyeli fonksiyon örnekleri (konsept):

lnmp.core.parse(text: str) -> Record
(Record Python objesi, altında PyLnmpRecord taşıyor)

lnmp.core.encode(record: Record) -> str

lnmp.core.encode_binary(record: Record) -> bytes

lnmp.envelope.wrap(record: Record, source: str, ..., trace_id: str | None) -> Envelope

lnmp.net.should_send_to_llm(envelope: Envelope, *, threshold: float = 0.7) -> bool

içinde context_score + routing_decide kullanıyor.

lnmp.llm.normalize_and_route(text: str, source: str) -> { record, envelope, decision, score }

Bu sayede Python geliştirici:

“LNMP nedir?” detayıyla boğuşmadan,

parse, encode, wrap, route fonksiyonlarıyla çalışabilecek,

İsteyen low-level lnmp_core_py’e iner, isteyen high-level lnmp ile takılır.

4. Neden Baştan Python Yazmıyoruz?

Ekip için net argümanlar:

Tek kaynak, tek doğruluk noktası:

Protokol davranışı, determinism, semantic checksum, SFE, routing kuralları:

Hepsi Rust’ta lnmp meta crate’te.

Python/TS/Go hepsi sadece farklı dil binding’i olacak.

Protokol değişikliği = Rust’ta yapılacak, binding’ler sadece yeni fonksiyonu kullanacak.

Bakım maliyeti:

Eğer Python’da codec, checksum, SFE, net mantığını yeniden yazarsak:

2 protokol implementasyonu olacak,

Her bugfix iki yerde,

Versiyon drift riski yüksek.

Stabilite ve determinism:

Rust’ta zaten:

sorted_fields, canonical encoding,

SemanticChecksum, ECO routing,

lnmp-net davranışı kilitli.

Python, pyo3 üzerinden bire bir aynısını kullanınca:

LLM testleri, CLI testleri, lnmp-net testleriyle uyum garanti.

Performans:

Özellikle:

büyük record set’leri,

embedding delta/quant,

spatial snapshot/delta

gibi işlerde Python saf implementasyon yerine:

Rust + native extension fark yaratır.

5. Nasıl Paketlenecek ve Yayınlanacak?

Build / dağıtım planı (özet):

lnmp-sdk-python (Rust + pyo3)

maturin build ile wheel üret:

lnmp_py_core-0.5.7-cp311-cp311-manylinux...whl vs.

CI pipeline:

Linux/Mac/Windows matrix → GH Actions.

PyPI’de private/public publish:

Paket adı: lnmp-sdk-python (veya benzeri).

lnmp Python paketi (pure Python wrapper)

pyproject.toml içinde lnmp-sdk-python’a dependency:

requires = ["lnmp-sdk-python>=0.5.7,<0.6"]

pip install lnmp → otomatik olarak lnmp-sdk-python çekilecek.

Versiyonlama:

Rust lnmp meta crate 0.5.7 ise, Python lnmp 0.5.7 veya 0.5.7.x gibi tutulabilir.

Version policy (ekip için net kural):

Major-minor senkron:

Rust lnmp 0.5.x → Python lnmp 0.5.x

Patch’ler:

Python patch’leri UI/ergonomi tarafında olabilir,

Core davranış değişirse Rust tarafında bump ile birlikte yapılır.

6. Kullanım Senaryosu (Mühendis Gözüyle)

Ekipteki Python geliştirici perspektifi:

from lnmp import core, envelope, net

# 1) LNMP text parse
record = core.parse("F12=14532;F7=1")

# 2) Binary encode (örneğin Kafka / NATS için)
binary = core.encode_binary(record)

# 3) Envelope ile zenginleştirme
env = envelope.wrap(
    record,
    source="health-service",
    timestamp_ms=None,   # otomatik now
    trace_id="trace-abc-123"
)

# 4) Routing + context scoring (ECO profil)
score = net.context_score(env)
decision = net.routing_decide(env)

if net.should_send_to_llm(env, threshold=0.7):
    # LLM'e gönder
    ...
else:
    # Local processing veya log
    ...


Bu kodun içindeki ağır işler:

parse, encode_binary, envelope.wrap, context_score, routing_decide
→ Hepsi Rust lnmp meta crate içinden geliyor.

Python ekibin bilmesi gereken tek şey:

“Bu SDK, LNMP protokolünün Python’dan erişilen yüzü.
Altında tüm mantık Rust’ta, deterministik ve testli.”

7. Ekip İçin Özet Mesaj

Mühendislere şu cümlelerle özetleyebilirsin:

LNMP’nin tek canonical implementasyonu Rust’ta (lnmp meta crate).

Python SDK’yı:

Rust’taki bu çekirdeğe bağlanan native binding (pyo3) olarak yapacağız,

Üzerine Pythonic wrapper’lar ekleyip,

parse/encode/envelope/net/routing/sfe gibi core işlevleri doğrudan kullanacağız.

Hiçbir kritik algoritmayı Python’da yeniden yazmayacağız:

Böylece davranış drift’i olmayacak,

Bakım tek yerden yürütülecek.

Dağıtımda:

lnmp-sdk-python (Rust native extension, pyo3 + maturin),

lnmp (high-level Python paket),

Versioning Rust meta crate ile uyumlu olacak.

Bu yapı:

Hem stabil (Rust core),

Hem performant (native extension),

Hem de geliştirici dostu (Pythonic API) bir SDK sağlar.