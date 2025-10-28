from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List
import pdfplumber
from langchain_core.documents import Document
import logging

app = FastAPI(title="PDF Loader Service")
logger = logging.getLogger("uvicorn")

class LoadPdfRequest(BaseModel):
    file_path: str

class LoadPdfResponse(BaseModel):
    pages: List[str]  # 每页纯文本

@app.post("/pdf/load")
async def load_pdf(req: LoadPdfRequest):
    try:
        documents = []
        with pdfplumber.open(req.file_path) as pdf:
            for page in pdf.pages:
                text = page.extract_text()
                if text and text.strip():
                    documents.append(text)
        logger.info(f"Loaded {len(documents)} pages from {req.file_path}")
        # 直接返回字符串列表，与 Rust 客户端期望的格式匹配
        return documents
    except Exception as e:
        logger.error(f"PDF load failed: {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))