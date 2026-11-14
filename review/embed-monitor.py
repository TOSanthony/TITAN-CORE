from pathlib import Path
r=Path(__file__).resolve().parents[1];p=r/'src/ui.html';s=p.read_text();a=s.index('// BEGIN MONITOR CLIENT');b=s.index('// END MONITOR CLIENT',a)+len('// END MONITOR CLIENT');p.write_text(s[:a]+(r/'review/monitor-client.js').read_text().strip()+s[b:])
