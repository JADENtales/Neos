"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export default function Home() {
  const names = ["全体", "一般", "耳打ち", "チーム", "クラブ", "システム", "叫び"];
  const colors = ["white", "white", "orange", "cyan", "violet", "yellow", "lightgreen"];
  const init = useRef(false);
  const refs = [
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
    useRef((null as unknown) as HTMLTextAreaElement),
  ];
  const [messages, setMessages] = useState([...Array(names.length)].map(_ => ""));
  const [views, setViews] = useState([...Array(names.length)].map(_ => true));
  const [verbose, setVerbose] = useState(false);
  const [wrap, setWrap] = useState("soft");
  const [vertical, setVertical] = useState(true);
  const [autoScroll, setAutoScroll] = useState(true);
  type State = [boolean, string, boolean, boolean];

  useEffect(() => {
    const f = async () => {
      for (let i = 0; i < names.length; ++i) {
        await listen('view' + i, async event => {
          const views = await invoke("get_views") as [boolean, boolean, boolean, boolean, boolean, boolean, boolean];
          setViews(views);
        });
      }
      await listen('verbose', async event => {
        const states = await invoke("get_states") as State;
        setVerbose(states[0]);
      });
      await listen('wrap', async event => {
        const states = await invoke("get_states") as State;
        setWrap(states[1]);
      });
      await listen('vertical', async event => {
        const states = await invoke("get_states") as State;
        setVertical(states[2]);
      });
      await listen('auto_scroll', async event => {
        const states = await invoke("get_states") as State;
        setAutoScroll(states[3]);
      });
      const views = await invoke("get_views") as [boolean, boolean, boolean, boolean, boolean, boolean, boolean];
      setViews(views);
      const states = await invoke("get_states") as State;
      setVerbose(states[0]);
      setWrap(states[1]);
      setVertical(states[2]);
      setAutoScroll(states[3]);
    };
    if (!init.current) {
      init.current = true;
      f();
    }
    const id = setInterval(async () => {
      const msgs = await invoke("read_log") as [string, string, string][][];
      for (let i = 0; i < msgs.length; ++i) {
        const msg = msgs[i].map(e => {
          if (verbose) {
            return e[2] + " " + e[0];
          }
          return e[0];
        }).join("\n");
        setMessages(prev => prev.map((e, j) => i === j ? msg : e));
        if (!autoScroll) {
          continue;
        }
        const ref = refs[i].current;
        if (ref !== null) {
          ref.scrollTop = ref.scrollHeight;
        }
      }
    }, 100);
    return () => clearInterval(id);
  }, [verbose, autoScroll]);

  return (
    <div className="container-fluid mt-1">
      {vertical && names.map((name, i) => {
        return views[i] && (
          <div className="mb-1" key={name}>
            <label className="form-label">{name}</label>
            <textarea className={"form-control " + styles.textarea} style={{"color": colors[i]}} value={messages[i]} rows={3} onChange={_ => {}} wrap={wrap} readOnly ref={refs[i]}></textarea>
          </div>
        );
      })}
      {!vertical &&
        <div className="row">
          {names.map((name, i) => {
            return views[i] && (
              <div className="col" key={name}>
                <label className="form-label">{name}</label>
                <textarea className={`form-control ${styles.textarea} ${styles.horizontal}`} style={{"color": colors[i]}} value={messages[i]} rows={3} onChange={_ => {}} wrap={wrap} readOnly ref={refs[i]}></textarea>
              </div>
            );
          })}
        </div>
      }
    </div>
  );
}
