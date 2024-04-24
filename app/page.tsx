"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export default function Home() {
  const names = ["All", "Public", "Private", "Team", "Club", "System", "Server"];
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
  const [verbose, setVerbose] = useState(false);
  const [wrap, setWrap] = useState("soft");
  const [vertical, setVertical] = useState(true);
  const [autoScroll, setAutoScroll] = useState(true);
  type State = [boolean, string, boolean, boolean];

  useEffect(() => {
    const f = async () => {
      await listen('verbose', async event => {
        const state = await invoke("get_state") as State;
        setVerbose(state[0]);
      });
      await listen('wrap', async event => {
        const state = await invoke("get_state") as State;
        setWrap(state[1]);
      });
      await listen('vertical', async event => {
        const state = await invoke("get_state") as State;
        setVertical(state[2]);
      });
      await listen('auto_scroll', async event => {
        const state = await invoke("get_state") as State;
        setAutoScroll(state[3]);
      });
      const state = await invoke("get_state") as State;
      setVerbose(state[0]);
      setWrap(state[1]);
      setVertical(state[2]);
      setAutoScroll(state[3]);
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
    <div className="container-fluid">
      {vertical && names.map((name, i) => {
        return (
          <div key={name + "_message"}>
            <label htmlFor={name.toLowerCase() + "_message"} className="form-label">{name}</label>
            <textarea className="form-control" id={name.toLowerCase() + "_message"} value={messages[i]} rows={3} onChange={_ => {}} wrap={wrap} ref={refs[i]}></textarea>
          </div>
        );
      })}
      {!vertical &&
        <div className="row">
          {names.map((name, i) => {
            return (
              <div className="col" key={name + "_message"}>
                <label htmlFor={name.toLowerCase() + "_message"} className="form-label">{name}</label>
                <textarea className={"form-control " + styles.textarea} id={name.toLowerCase() + "_message"} value={messages[i]} rows={3} onChange={_ => {}} wrap={wrap} ref={refs[i]}></textarea>
              </div>
            );
          })}
        </div>
      }
    </div>
  );
}
