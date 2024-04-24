"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { emit, listen } from '@tauri-apps/api/event'

export default function Home() {
  const names = ["All", "Public", "Private", "Team", "Club", "System", "Server"];
  const init = useRef(false);
  const [messages, setMessages] = useState([...Array(names.length)].map(_ => ""));

  useEffect(() => {
    const f = async () => {
      await listen('time', (event) => {
      });
    };
    if (!init.current) {
      init.current = true;
      f();
    }
    const id = setInterval(async () => {
      const msgs = await invoke("read_log") as [string, string, string][][];
      for (let i = 0; i < msgs.length; ++i) {
        const msg = msgs[i].map(e => e[2] + e[0]).join("\n");
        setMessages(prev => prev.map((e, j) => i === j ? msg : e));
      }
    }, 500);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="container-fluid">
      {true && names.map((name, i) => {
        return (
          <div key={name + "_message"}>
            <label htmlFor={name.toLowerCase() + "_message"} className="form-label">{name}</label>
            <textarea className="form-control" id={name.toLowerCase() + "_message"} value={messages[i]} rows={3} onChange={_ => {}}></textarea>
          </div>
        );
      })}
      {false &&
        <div className="row">
          {names.map((name, i) => {
            return (
              <div className="col" key={name + "_message"}>
                <label htmlFor={name.toLowerCase() + "_message"} className="form-label">{name}</label>
                <textarea className={"form-control " + styles.textarea} id={name.toLowerCase() + "_message"} value={messages[i]} rows={3} onChange={_ => {}}></textarea>
              </div>
            );
          })}
        </div>
      }
    </div>
  );
}
