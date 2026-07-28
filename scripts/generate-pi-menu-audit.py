#!/usr/bin/env python3
import hashlib, json, re
from pathlib import Path

root=Path(__file__).resolve().parents[1]
commands_path=root/'apps/pi-extension/src/commands.ts'
session_path=root/'apps/pi-extension/src/session.ts'
out=root/'docs/contracts/48-49-focusa-pi-menu-inventory.json'
source=commands_path.read_text()
commands=[]
for match in re.finditer(r'pi\.registerCommand\("([^"]+)",\s*\{\s*description:\s*(?:"([^"]+)"|`([^`]+)`)',source):
    name=match.group(1); desc=match.group(2) or match.group(3)
    commands.append({
      'id':name,'kind':'command','owner':'focusa-pi-extension','user_job':desc,
      'placement':'top_level_command','rationale':'direct operator action or compatibility entrypoint',
      'help':desc,'scope':'attachment_or_explicit_project','risk':'mutation_gated_by_handler',
      'interaction_test':'Pi typecheck plus command-specific contract test'
    })
constants={
    match.group(1): re.findall(r'"([^"]+)"', match.group(2))
    for match in re.finditer(r'const\s+([A-Z_]+)\s*=\s*\[([^\]]+)\]', source)
}
def setting_items(marker,end):
    text=source[source.index(marker):source.index(end,source.index(marker))]
    items=[]
    starts=list(re.finditer(r'\{\s*id:\s*"([^"]+)",\s*label:\s*"([^"]+)"',text))
    for i,m in enumerate(starts):
      block=text[m.start():(starts[i+1].start() if i+1<len(starts) else len(text))]
      vm=re.search(r'values:\s*(\[[^\]]*\]|[A-Z_]+)',block,re.S)
      raw=vm.group(1) if vm else '[]'
      values=re.findall(r'"([^"]+)"',raw) if raw.startswith('[') else constants.get(raw,[])
      items.append((m.group(1),m.group(2),values))
    return items
simple=setting_items('const buildSimpleItems','const buildAdvancedItems')
advanced=setting_items('const buildAdvancedItems','await ctx.ui.custom')
simple_ids={item[0] for item in simple}
settings=[]
for placement,items in [('simple_default',simple),('advanced_searchable',advanced)]:
  for ident,label,values in items:
    settings.append({
      'id':ident,'kind':'setting','owner':'focusa-config','user_job':label,
      'placement':placement,'canonical_home':'simple_default' if ident in simple_ids else 'advanced_searchable',
      'rationale':'common workflow' if ident in simple_ids else 'expert tuning kept out of default menu',
      'help':label,'scope':'project_config','risk':'persistent_config_write','values':values,
      'interaction_test':'settings attachment safety + exhaustive value callback gate'
    })
prompts=[
 {'id':'project_trust_confirm','kind':'confirm','owner':'focusa-session','user_job':'Confirm project trust before activation'},
 {'id':'project_root_select','kind':'select','owner':'focusa-session','user_job':'Choose one verified project root'},
 {'id':'continuity_select','kind':'select','owner':'focusa-session','user_job':'Choose bounded continuation scope'},
]
for row in prompts:
 row.update({'placement':'contextual_only','rationale':'shown only when authority is ambiguous','help':row['user_job'],
             'scope':'session','risk':'authority_selection','interaction_test':'session interaction contract'})
payload={
 'schema':'focusa.pi_menu_inventory.v1','generated':True,
 'source_sha256':hashlib.sha256((source+session_path.read_text()).encode()).hexdigest(),
 'baseline':{'top_level_commands':9,'simple_settings':11,'advanced_settings':29,'simple_choice_depth':11},
 'after':{'top_level_commands':len(commands),'simple_settings':len(simple),'advanced_settings':len(advanced),'simple_choice_depth':len(simple)},
 'items':commands+settings+prompts,
 'migration':{
   'otaProfile':'advanced settings search','workLoopStatusHeartbeatMs':'advanced settings search',
   'vitalInfoPromptSurfaces':'advanced settings search'
 }
}
out.write_text(json.dumps(payload,indent=2)+'\n')
print(f"generated {out.relative_to(root)}: {len(payload['items'])} interactive entries")
