// chanceify™ song page: a Cloudflare Worker that makes the link "Copy the song
// for Discord" gives out. Discord, and anyone who opens it, sees the cover,
// the song, and a button that opens it in Spotify. No keys, no secrets.
//
//   https://<your-worker>.workers.dev/t/<spotify track id>?t=<title>&a=<artist>
//
// The cover comes from Spotify's public oEmbed. The title and artist come from
// the link itself (chanceify puts them there), with oEmbed as the fallback.

// chance's little bear. Paste the picture here as a data address
// ("data:image/png;base64,...") and it shows after every "chance" link.
const BEAR = "";

const esc = (s) =>
  String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const match = url.pathname.match(/^\/t\/([A-Za-z0-9]{10,30})\/?$/);
    if (!match) {
      return new Response(page({ title: "chanceify™", artist: "A Spotify player by chance", id: null }), html());
    }
    const id = match[1];
    let title = (url.searchParams.get("t") || "").slice(0, 150);
    const artist = (url.searchParams.get("a") || "").slice(0, 150);
    let image = "";
    try {
      const res = await fetch(
        "https://open.spotify.com/oembed?url=" + encodeURIComponent("https://open.spotify.com/track/" + id),
        { cf: { cacheTtl: 3600, cacheEverything: true } },
      );
      if (res.ok) {
        const data = await res.json();
        image = data.thumbnail_url || "";
        if (!title) title = data.title || "";
      }
    } catch (_) {}
    return new Response(page({ title: title || "A song", artist, id, image }), html());
  },
};

const html = () => ({ headers: { "content-type": "text/html; charset=utf-8", "cache-control": "public, max-age=600" } });

function page({ title, artist, id, image }) {
  const spotify = id ? `https://open.spotify.com/track/${id}` : "https://github.com/madlygeeked/chanceify";
  const line = artist ? `${artist}` : "Listening on chanceify™";
  const desc = artist ? `${artist}. Listening on chanceify™` : "Listening on chanceify™";
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${esc(title)} - chanceify™</title>
<meta name="theme-color" content="#8b5cf6">
<meta property="og:type" content="music.song">
<meta property="og:site_name" content="chanceify™">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(desc)}">
${image ? `<meta property="og:image" content="${esc(image)}">` : ""}
<meta property="og:url" content="${esc(spotify)}">
<meta name="twitter:card" content="summary_large_image">
${image ? `<meta name="twitter:image" content="${esc(image)}">
<meta property="og:image:width" content="640">
<meta property="og:image:height" content="640">` : ""}
<style>
:root{color-scheme:dark}
*{box-sizing:border-box}
body{margin:0;min-height:100vh;display:grid;place-items:center;padding:24px;font-family:system-ui,"Segoe UI",sans-serif;color:#f3f0ff;
background:radial-gradient(900px 600px at 20% 10%,#4c1d95 0,transparent 60%),radial-gradient(800px 600px at 90% 90%,#312e81 0,transparent 60%),#0f0b1d}
.card{width:min(420px,100%);text-align:center}
.cover{width:100%;aspect-ratio:1;border-radius:22px;background:#241a45 center/cover;box-shadow:0 30px 80px #0009,0 0 0 1px #ffffff1a}
h1{font-size:26px;margin:22px 0 4px;line-height:1.2}
p{margin:0;color:#c4b5fd;font-size:16px}
.btn{display:block;text-align:center;padding:13px 28px;border-radius:999px;background:linear-gradient(90deg,#6366f1,#8b5cf6);color:#fff;font-weight:600;text-decoration:none}
.btn:hover{filter:brightness(1.12)}
.row{margin-top:14px;display:flex;gap:10px;justify-content:center;flex-wrap:wrap}
.stack{margin-top:24px;flex-direction:column;align-items:stretch}
.btn.alt{background:#ffffff14;border:1px solid #a78bfa55;color:#ede9fe;font-weight:500}
.btn.alt:hover{background:#a78bfa2a}
.play{margin-top:18px;border:0;border-radius:14px;color-scheme:normal}
.bear{height:1.5em;width:auto;vertical-align:middle;margin-left:6px}
#swirl{position:fixed;inset:0;width:100%;height:100%;z-index:-1}
.card{position:relative}
.foot{margin-top:28px;font-size:13px;color:#a78bfa99}
.foot a{color:inherit}
</style></head><body>
<canvas id="swirl"></canvas>
<main class="card">
<div class="cover"${image ? ` style="background-image:url('${esc(image)}')"` : ""}></div>
<h1>${esc(title)}</h1>
<p>${esc(line)}</p>
${id ? `<div class="row stack"><a class="btn" href="chanceify://track/${esc(id)}">Open in chanceify</a>
<a class="btn alt" href="${esc(spotify)}">Listen on Spotify</a>
<a class="btn alt" href="spotify:track:${esc(id)}">Open in the Spotify app</a></div>
<iframe class="play" title="Play a preview" src="https://open.spotify.com/embed/track/${esc(id)}?theme=0" width="100%" height="152" frameborder="0" allow="autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture" loading="lazy"></iframe>` : ""}
<div class="foot">Built with love by <a href="https://github.com/madlygeeked">chance</a>${BEAR ? `<img class="bear" src="${BEAR}" alt="">` : ""}</div>
</main>
<script>
// A swirl in the cover's own colours, drifting like chanceify's visualizer.
(function(){
  var cv=document.getElementById("swirl"),g=cv.getContext("2d");
  var cols=[[139,92,246],[99,102,241],[168,85,247]];
  var W,H;function size(){W=cv.width=innerWidth;H=cv.height=innerHeight}size();addEventListener("resize",size);
  var still=matchMedia("(prefers-reduced-motion: reduce)").matches;
  var url=${JSON.stringify(image || "")};
  if(url){
    var im=new Image();im.crossOrigin="anonymous";
    im.onload=function(){try{
      var c=document.createElement("canvas");c.width=c.height=24;var x=c.getContext("2d");x.drawImage(im,0,0,24,24);
      var d=x.getImageData(0,0,24,24).data,bins={};
      for(var i=0;i<d.length;i+=4){
        var r=d[i],gg=d[i+1],b=d[i+2],mx=Math.max(r,gg,b),mn=Math.min(r,gg,b);
        if(mx<50||mx-mn<28)continue;
        var k=(r>>6)+","+(gg>>6)+","+(b>>6),o=bins[k]||(bins[k]=[0,0,0,0]);
        o[0]+=r;o[1]+=gg;o[2]+=b;o[3]++;
      }
      var list=Object.values(bins).sort(function(a,b){return b[3]-a[3]}).slice(0,4)
        .map(function(o){return[o[0]/o[3]|0,o[1]/o[3]|0,o[2]/o[3]|0]});
      if(list.length)cols=list;
    }catch(e){}};
    im.src=url;
  }
  var t=0;
  function frame(){
    g.globalCompositeOperation="source-over";
    g.fillStyle="rgba(15,11,29,"+(still?1:0.12)+")";g.fillRect(0,0,W,H);
    g.globalCompositeOperation="lighter";
    var cx=W/2,cy=H/2,R=Math.hypot(W,H)/2;
    for(var a=0;a<cols.length*2;a++){
      var c=cols[a%cols.length];
      for(var i=0;i<46;i++){
        var f=i/46,ang=t*0.35*(a%2?-1:1)+f*7+a*1.7,rad=f*R;
        var px=cx+Math.cos(ang)*rad,py=cy+Math.sin(ang)*rad*0.8;
        var s=14+f*70;
        var rg=g.createRadialGradient(px,py,0,px,py,s);
        rg.addColorStop(0,"rgba("+c[0]+","+c[1]+","+c[2]+","+(0.10*(1-f*0.5))+")");
        rg.addColorStop(1,"rgba("+c[0]+","+c[1]+","+c[2]+",0)");
        g.fillStyle=rg;g.fillRect(px-s,py-s,s*2,s*2);
      }
    }
    t+=0.016;
    if(!still)requestAnimationFrame(frame);
  }
  if(still){for(var n=0;n<30;n++){t+=0.2;frame()}}else frame();
})();
</script>
</body></html>`;
}
