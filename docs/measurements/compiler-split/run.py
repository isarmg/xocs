import json,os,resource,subprocess,sys,time
from pathlib import Path
root,label,*packages=sys.argv[1:]
base=Path(os.environ.get('COMPILER_BENCH_OUTPUT', '/workspace/.tooling/bench-compiler'))
base.mkdir(parents=True, exist_ok=True)
log=base/(label+'.log')
start=time.monotonic();before=resource.getrusage(resource.RUSAGE_CHILDREN);peak=0
with log.open('w') as stream:
 p=subprocess.Popen(['cargo','+1.99.0','build','--locked','--offline','--jobs','2',*[arg for package in packages for arg in ['-p',package]]],cwd=root,stdout=stream,stderr=subprocess.STDOUT)
 while p.poll() is None:
  all_processes={}
  for f in Path('/proc').glob('[0-9]*/stat'):
   try:
    values=f.read_text().rsplit(')',1)[1].split();all_processes[int(f.parent.name)]=(int(values[1]),int(values[21])*os.sysconf('SC_PAGE_SIZE'))
   except (OSError,ValueError,IndexError):pass
  children={p.pid};total=0
  while True:
   found={pid for pid,(ppid,rss) in all_processes.items() if ppid in children}-children
   if not found:break
   children|=found
  peak=max(peak,sum(all_processes.get(pid,(0,0))[1] for pid in children))
  time.sleep(.05)
 after=resource.getrusage(resource.RUSAGE_CHILDREN)
 result={'label':label,'root':root,'packages':packages,'jobs':2,'elapsed_seconds':round(time.monotonic()-start,3),'cpu_user_seconds':round(after.ru_utime-before.ru_utime,3),'cpu_system_seconds':round(after.ru_stime-before.ru_stime,3),'max_process_rss_kib':after.ru_maxrss,'sampled_tree_rss_bytes':peak,'status':p.returncode}
 (base/(label+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));sys.exit(p.returncode)
